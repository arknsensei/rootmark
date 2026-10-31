//! Fixed-order BERT forward pass (P2-V.1, K-P2V.3 T1 (a); candidate (r) of
//! `reports/rounds/P2-V.1/e0/feasibility.md`).
//!
//! The same `model.onnx` weights, the same inputs and the same graph as the
//! ONNX Runtime session of `crate::embed`, computed so that every output
//! component is defined by one sequence of IEEE-754 binary32/binary64
//! operations, independent of the processor:
//! - every sum runs in a fixed order (index 0, 1, 2, …) with one accumulator
//!   per output component; independent components may share SIMD lanes (the
//!   compiler vectorizes across components, never inside one sum);
//! - products are accumulated with fused multiply-add (`f32::mul_add`: one
//!   rounding, defined by IEEE 754, the same in hardware on x86_64 FMA,
//!   aarch64 and in software), every other operation is a plain binary32
//!   operation (Rust never contracts `a * b + c` by itself); no
//!   reassociation; `sqrt` and division are correctly rounded everywhere;
//! - `exp` and `erf` are computed here from `+ - * /` in binary64
//!   ([`exp64`], [`erf64`]) — never the platform's `libm` — then rounded to
//!   binary32 where the graph has a binary32 tensor;
//! - no threads (`index/CLAUDE.md`).
//!
//! The graph (both manifest models, `minilm-l6-v2` and
//! `multilingual-minilm-l12-v2`, opset 14, Hugging Face `BertModel` export):
//! embeddings `(word + token_type) + position` → LayerNorm; per layer
//! Q/K/V = `x·W + b`, scores = `(q·s)·(k·s)` with `s = √(1/√d_head)`,
//! `+ mask` (0 without padding), softmax, `·V`, output dense `+ x` →
//! LayerNorm, intermediate dense → GELU `(x·(erf(x/√2)+1))·0.5`, output dense
//! `+ h` → LayerNorm (ε = 1e-12). Loading checks the weights by name and
//! shape and the MatMul nodes by name; any other graph is an error.

use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::path::Path;

// ln 2 = LN2_HI + LN2_LO, LN2_HI with 32 trailing zero bits (k·LN2_HI exact)
const LN2_HI: f64 = f64::from_bits(0x3FE6_2E42_FEE0_0000); // 6.93147180369123816490e-01
const LN2_LO: f64 = f64::from_bits(0x3DEA_39EF_3579_3C76); // 1.90821492927058770002e-10
/// Nearest integer, ties to even, for |y| < 2⁵¹: `(y + 1.5·2⁵²) − 1.5·2⁵²`
/// (two correctly rounded additions; vectorizes, unlike `f64::round`).
#[inline(always)]
fn round_even(y: f64) -> f64 {
    const M: f64 = 6_755_399_441_055_744.0;
    (y + M) - M
}

/// `1/i!`, i = 0..=13.
const INV_FACT: [f64; 14] = [
    1.0,
    1.0,
    1.0 / 2.0,
    1.0 / 6.0,
    1.0 / 24.0,
    1.0 / 120.0,
    1.0 / 720.0,
    1.0 / 5_040.0,
    1.0 / 40_320.0,
    1.0 / 362_880.0,
    1.0 / 3_628_800.0,
    1.0 / 39_916_800.0,
    1.0 / 479_001_600.0,
    1.0 / 6_227_020_800.0,
];

/// `e^x` in binary64 from `+ - * /` only (range reduction by `ln 2` in two
/// parts, Taylor polynomial of degree 13 on |r| ≤ ½ ln 2, exact scaling by
/// 2^k). Relative error below 1e-15; identical on every IEEE-754 machine.
pub fn exp64(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if x > 709.0 {
        return f64::INFINITY;
    }
    if x < -700.0 {
        return 0.0;
    }
    let k = round_even(x * std::f64::consts::LOG2_E);
    let r = (x - k * LN2_HI) - k * LN2_LO;
    // Σ r^i / i!, i = 0..=13, Horner from the highest degree
    let mut p = INV_FACT[13];
    for c in INV_FACT[..13].iter().rev() {
        p = p * r + c;
    }
    // 2^k, k in [-1010, 1023]: a normal binary64
    let scale = f64::from_bits(((k as i64 + 1023) as u64) << 52);
    p * scale
}

/// `erf(x)` in binary64 from `+ - * /` and [`exp64`], slow reference:
/// `erf(a) = 2/√π · e^(−a²) · Σₙ (2a²)ⁿ · a / (1·3·…·(2n+1))` (all terms
/// positive, no cancellation), odd in `x`, ±1 for |x| ≥ 6 (1 − erf(6) < 3e-17).
/// Builds the table of [`erf64`].
pub fn erf64_series(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    let a = x.abs();
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    if a >= 6.0 {
        return sign;
    }
    let a2 = a * a;
    let two_a2 = 2.0 * a2;
    let mut term = a;
    let mut sum = a;
    let mut n = 0usize;
    while n < 200 {
        n += 1;
        term = term * two_a2 * INV_ODD[n];
        sum += term;
        if term <= sum * 1e-17 {
            break;
        }
    }
    let v = std::f64::consts::FRAC_2_SQRT_PI * exp64(-a2) * sum;
    sign * if v > 1.0 { 1.0 } else { v }
}

/// Grid of [`erf64`]: points `x₀ = i/64`, i = 0..=384 (0 … 6).
const ERF_STEPS: f64 = 64.0;
const ERF_POINTS: usize = 385;
/// Taylor terms per grid point (degree of the polynomial in `h`).
const ERF_DEG: usize = 8;

/// Per grid point `x₀` the coefficients of `erf(x₀ + h) = Σₖ cₖ hᵏ`, k ≤ 8:
/// `c₀ = erf(x₀)` ([`erf64_series`]), `cₖ₊₁ = 2/√π · e^(−x₀²) · (−1)ᵏ Hₖ(x₀) / (k+1)!`
/// (Hermite polynomials, `Hₖ₊₁ = 2x Hₖ − 2k Hₖ₋₁`).
fn erf_table() -> &'static [[f64; ERF_DEG + 1]; ERF_POINTS] {
    static T: std::sync::OnceLock<Box<[[f64; ERF_DEG + 1]; ERF_POINTS]>> =
        std::sync::OnceLock::new();
    T.get_or_init(|| {
        let mut t = Box::new([[0f64; ERF_DEG + 1]; ERF_POINTS]);
        for (i, row) in t.iter_mut().enumerate() {
            let x0 = i as f64 / ERF_STEPS;
            row[0] = erf64_series(x0);
            let g = std::f64::consts::FRAC_2_SQRT_PI * exp64(-(x0 * x0));
            let (mut hm1, mut h) = (0f64, 1f64); // H₋₁ (unused), H₀
            let mut fact = 1f64; // (k+1)!
            for k in 0..ERF_DEG {
                fact *= (k + 1) as f64;
                let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
                row[k + 1] = g * sign * h / fact;
                let next = 2.0 * x0 * h - 2.0 * k as f64 * hm1;
                hm1 = h;
                h = next;
            }
        }
        t
    })
}

/// `erf(x)` in binary64: the nearest grid point of [`erf_table`] and its
/// Taylor polynomial in `h = |x| − x₀` (|h| ≤ 1/128; truncation below 1e-15),
/// Horner from the highest degree; odd in `x`, ±1 for |x| ≥ 6.
pub fn erf64(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    let a = x.abs();
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    if a >= 6.0 {
        return sign;
    }
    let i = (a * ERF_STEPS).round() as usize;
    let row = &erf_table()[i.min(ERF_POINTS - 1)];
    let h = a - i as f64 / ERF_STEPS;
    let mut p = row[ERF_DEG];
    for c in row[..ERF_DEG].iter().rev() {
        p = p * h + c;
    }
    sign * if p > 1.0 { 1.0 } else { p }
}

/// `1/(2n+1)` for n = 0..=200 (binary64, correctly rounded at compile time).
const INV_ODD: [f64; 201] = {
    let mut t = [0f64; 201];
    let mut n = 0;
    while n < 201 {
        t[n] = 1.0 / (2 * n + 1) as f64;
        n += 1;
    }
    t
};

/// A dense layer `y = x·W + b`: `W` (`[din, dout]` row-major in the graph)
/// stored as strips of `TC` columns, each strip `din` rows of `TC` values,
/// so that a tile reads its weights contiguously. Layout only: every output
/// component is the same sum.
struct Dense {
    din: usize,
    dout: usize,
    strips: Vec<f32>,
    bias: Vec<f32>,
}

impl Dense {
    fn new(w: Vec<f32>, din: usize, bias: Vec<f32>) -> Result<Dense> {
        let dout = bias.len();
        if w.len() != din * dout || !dout.is_multiple_of(TC) {
            bail!("detbert: dense {din}×{dout} (columns must be a multiple of {TC})");
        }
        let mut strips = Vec::with_capacity(w.len());
        for s in 0..dout / TC {
            for i in 0..din {
                strips.extend_from_slice(&w[i * dout + s * TC..i * dout + s * TC + TC]);
            }
        }
        Ok(Dense {
            din,
            dout,
            strips,
            bias,
        })
    }
}

/// One encoder layer.
struct Layer {
    q: Dense,
    k: Dense,
    v: Dense,
    o: Dense,
    ln1_w: Vec<f32>,
    ln1_b: Vec<f32>,
    i: Dense,
    out: Dense,
    ln2_w: Vec<f32>,
    ln2_b: Vec<f32>,
}

/// Tile of the dense kernel: `TR` rows × `TC` columns of accumulators.
const TR: usize = 4;
const TC: usize = 16;

pub struct DetBert {
    hidden: usize,
    heads: usize,
    vocab: usize,
    max_pos: usize,
    word: Vec<f32>,
    pos: Vec<f32>,
    types: Vec<f32>,
    ln_w: Vec<f32>,
    ln_b: Vec<f32>,
    layers: Vec<Layer>,
}

const LN_EPS: f32 = 1e-12;

// ---------------------------------------------------------------- protobuf

struct Pb<'a> {
    b: &'a [u8],
    i: usize,
}

enum Field<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
    Fixed,
}

impl<'a> Pb<'a> {
    fn new(b: &'a [u8]) -> Self {
        Pb { b, i: 0 }
    }
    fn varint(&mut self) -> Result<u64> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let byte = *self.b.get(self.i).context("onnx: truncated varint")?;
            self.i += 1;
            v |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(v);
            }
        }
        bail!("onnx: varint too long")
    }
    fn next(&mut self) -> Result<Option<(u64, Field<'a>)>> {
        if self.i >= self.b.len() {
            return Ok(None);
        }
        let key = self.varint()?;
        let (no, wire) = (key >> 3, key & 7);
        let f = match wire {
            0 => Field::Varint(self.varint()?),
            1 => {
                self.skip(8)?;
                Field::Fixed
            }
            2 => {
                let n = usize::try_from(self.varint()?)?;
                let s = self
                    .b
                    .get(self.i..self.i + n)
                    .context("onnx: truncated field")?;
                self.i += n;
                Field::Bytes(s)
            }
            5 => {
                self.skip(4)?;
                Field::Fixed
            }
            w => bail!("onnx: unsupported wire type {w}"),
        };
        Ok(Some((no, f)))
    }
    fn skip(&mut self, n: usize) -> Result<()> {
        if self.i + n > self.b.len() {
            bail!("onnx: truncated fixed field");
        }
        self.i += n;
        Ok(())
    }
}

fn utf8(b: &[u8]) -> Result<String> {
    Ok(std::str::from_utf8(b)?.to_string())
}

struct Tensor {
    dims: Vec<usize>,
    data: Vec<f32>,
}

/// TensorProto: dims (1), data_type (2), name (8), raw_data (9); only
/// binary32 tensors with `raw_data` are read, anything else (external data,
/// `float_data`) is an error when the tensor is needed.
fn tensor(b: &[u8]) -> Result<(String, Option<Tensor>)> {
    let mut p = Pb::new(b);
    let (mut dims, mut dtype, mut name, mut raw) = (Vec::new(), 0u64, String::new(), None);
    while let Some((no, f)) = p.next()? {
        match (no, f) {
            (1, Field::Varint(v)) => dims.push(usize::try_from(v)?),
            (1, Field::Bytes(packed)) => {
                let mut q = Pb::new(packed);
                while q.i < packed.len() {
                    dims.push(usize::try_from(q.varint()?)?);
                }
            }
            (2, Field::Varint(v)) => dtype = v,
            (8, Field::Bytes(s)) => name = utf8(s)?,
            (9, Field::Bytes(s)) => raw = Some(s),
            _ => {}
        }
    }
    let t = match (dtype, raw) {
        (1, Some(r)) => {
            let n: usize = dims.iter().product();
            if r.len() != n * 4 {
                bail!("onnx: tensor {name}: {} bytes for {n} f32", r.len());
            }
            Some(Tensor {
                dims,
                data: r
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect(),
            })
        }
        _ => None,
    };
    Ok((name, t))
}

/// (initializers by name, MatMul node name → its initializer input).
type Graph = (HashMap<String, Tensor>, HashMap<String, String>);

fn read_graph(model: &[u8]) -> Result<Graph> {
    let mut p = Pb::new(model);
    let mut graph = None;
    while let Some((no, f)) = p.next()? {
        if let (7, Field::Bytes(g)) = (no, f) {
            graph = Some(g);
        }
    }
    let g = graph.context("onnx: no graph")?;
    let mut inits = HashMap::new();
    let mut matmuls = HashMap::new();
    let mut p = Pb::new(g);
    while let Some((no, f)) = p.next()? {
        match (no, f) {
            (5, Field::Bytes(t)) => {
                let (name, t) = tensor(t)?;
                if let Some(t) = t {
                    inits.insert(name, t);
                }
            }
            (1, Field::Bytes(n)) => {
                let mut q = Pb::new(n);
                let (mut inputs, mut name, mut op) = (Vec::new(), String::new(), String::new());
                while let Some((no, f)) = q.next()? {
                    match (no, f) {
                        (1, Field::Bytes(s)) => inputs.push(utf8(s)?),
                        (3, Field::Bytes(s)) => name = utf8(s)?,
                        (4, Field::Bytes(s)) => op = utf8(s)?,
                        _ => {}
                    }
                }
                if op == "MatMul" && inputs.len() == 2 {
                    matmuls.insert(name, inputs[1].clone());
                }
            }
            _ => {}
        }
    }
    Ok((inits, matmuls))
}

fn take(inits: &mut HashMap<String, Tensor>, name: &str, shape: &[usize]) -> Result<Vec<f32>> {
    let t = inits
        .remove(name)
        .ok_or_else(|| anyhow!("onnx: no f32 initializer {name}"))?;
    if t.dims != shape {
        bail!("onnx: {name} has shape {:?}, expected {shape:?}", t.dims);
    }
    Ok(t.data)
}

impl DetBert {
    /// Reads the weights of a manifest-verified `model.onnx`.
    pub fn load(model_onnx: &Path) -> Result<DetBert> {
        let bytes =
            std::fs::read(model_onnx).with_context(|| format!("read {}", model_onnx.display()))?;
        let (mut inits, matmuls) = read_graph(&bytes)?;
        let word_dims = {
            let t = inits
                .get("embeddings.word_embeddings.weight")
                .context("onnx: no word embeddings")?;
            t.dims.clone()
        };
        let [vocab, hidden] = word_dims[..] else {
            bail!("onnx: word embeddings of shape {word_dims:?}");
        };
        if hidden != crate::embed::DIM {
            bail!("onnx: hidden size {hidden}, expected {}", crate::embed::DIM);
        }
        let heads = 12;
        let max_pos = 512;
        let word = take(
            &mut inits,
            "embeddings.word_embeddings.weight",
            &[vocab, hidden],
        )?;
        let pos = take(
            &mut inits,
            "embeddings.position_embeddings.weight",
            &[max_pos, hidden],
        )?;
        let types = take(
            &mut inits,
            "embeddings.token_type_embeddings.weight",
            &[2, hidden],
        )?;
        let ln_w = take(&mut inits, "embeddings.LayerNorm.weight", &[hidden])?;
        let ln_b = take(&mut inits, "embeddings.LayerNorm.bias", &[hidden])?;
        let mut layers = Vec::new();
        let mut inter = 0;
        for l in 0.. {
            let pre = format!("encoder.layer.{l}.");
            if !matmuls.contains_key(&format!("/encoder/layer.{l}/attention/self/query/MatMul")) {
                if l == 0 {
                    bail!("onnx: no encoder layer");
                }
                break;
            }
            let mm = |part: &str| -> Result<String> {
                matmuls
                    .get(&format!("/encoder/layer.{l}/{part}/MatMul"))
                    .cloned()
                    .ok_or_else(|| anyhow!("onnx: layer {l}: no MatMul {part}"))
            };
            let wi_name = mm("intermediate/dense")?;
            let i_size = inits
                .get(&wi_name)
                .and_then(|t| t.dims.get(1).copied())
                .context("onnx: intermediate weight")?;
            if inter == 0 {
                inter = i_size;
            }
            let (h, i) = (hidden, inter);
            let ln = |s: &str| format!("{pre}{s}");
            let mut dense = |w: &str, b: &str, din: usize, dout: usize| -> Result<Dense> {
                let w = take(&mut inits, w, &[din, dout])?;
                Dense::new(w, din, take(&mut inits, &ln(b), &[dout])?)
            };
            let q = dense(
                &mm("attention/self/query")?,
                "attention.self.query.bias",
                h,
                h,
            )?;
            let k = dense(&mm("attention/self/key")?, "attention.self.key.bias", h, h)?;
            let v = dense(
                &mm("attention/self/value")?,
                "attention.self.value.bias",
                h,
                h,
            )?;
            let o = dense(
                &mm("attention/output/dense")?,
                "attention.output.dense.bias",
                h,
                h,
            )?;
            let im = dense(&wi_name, "intermediate.dense.bias", h, i)?;
            let out = dense(&mm("output/dense")?, "output.dense.bias", i, h)?;
            layers.push(Layer {
                q,
                k,
                v,
                o,
                ln1_w: take(&mut inits, &ln("attention.output.LayerNorm.weight"), &[h])?,
                ln1_b: take(&mut inits, &ln("attention.output.LayerNorm.bias"), &[h])?,
                i: im,
                out,
                ln2_w: take(&mut inits, &ln("output.LayerNorm.weight"), &[h])?,
                ln2_b: take(&mut inits, &ln("output.LayerNorm.bias"), &[h])?,
            });
        }
        Ok(DetBert {
            hidden,
            heads,
            vocab,
            max_pos,
            word,
            pos,
            types,
            ln_w,
            ln_b,
            layers,
        })
    }

    pub fn layers(&self) -> usize {
        self.layers.len()
    }

    /// `last_hidden_state` `[n, hidden]` row-major for one unpadded text.
    pub fn forward(&self, ids: &[u32], type_ids: &[u32]) -> Result<Vec<f32>> {
        self.forward_impl(ids, type_ids)
    }

    #[inline(always)]
    fn forward_impl(&self, ids: &[u32], type_ids: &[u32]) -> Result<Vec<f32>> {
        let (n, h) = (ids.len(), self.hidden);
        if n == 0 || n > self.max_pos || type_ids.len() != n {
            bail!("detbert: {n} tokens (max {})", self.max_pos);
        }
        let mut x = vec![0f32; n * h];
        for t in 0..n {
            let id = ids[t] as usize;
            let ty = type_ids[t] as usize;
            if id >= self.vocab || ty >= 2 {
                bail!("detbert: token id {id} / type {ty} out of range");
            }
            let w = &self.word[id * h..(id + 1) * h];
            let ty = &self.types[ty * h..(ty + 1) * h];
            let p = &self.pos[t * h..(t + 1) * h];
            let row = &mut x[t * h..(t + 1) * h];
            for j in 0..h {
                row[j] = (w[j] + ty[j]) + p[j];
            }
        }
        layer_norm(&mut x, h, &self.ln_w, &self.ln_b);
        for l in &self.layers {
            x = self.layer(l, &x, n);
        }
        Ok(x)
    }

    #[inline(always)]
    fn layer(&self, l: &Layer, x: &[f32], n: usize) -> Vec<f32> {
        let (h, nh) = (self.hidden, self.heads);
        let dh = h / nh;
        let q = dense(x, n, &l.q);
        let k = dense(x, n, &l.k);
        let v = dense(x, n, &l.v);
        // s = √(1/√d_head), as the graph computes it in binary32
        let s = (1.0f32 / (dh as f32).sqrt()).sqrt();
        let mut ctx = vec![0f32; n * h];
        // columns of the score matrix, padded to whole strips (padding
        // columns are zero weights; their outputs are never read)
        let np = n.div_ceil(TC) * TC;
        let mut qs = vec![0f32; n * dh];
        let mut kstrips = vec![0f32; np * dh];
        let mut vstrips = vec![0f32; n * dh];
        let mut scores = vec![0f32; n * np];
        let mut hctx = vec![0f32; n * dh];
        for hd in 0..nh {
            let off = hd * dh;
            for i in 0..n {
                for d in 0..dh {
                    qs[i * dh + d] = q[i * h + off + d] * s;
                }
            }
            // kstrips: strip c0 = column block j ∈ [c0·TC, c0·TC+TC), row d
            for (st, strip) in kstrips.chunks_exact_mut(dh * TC).enumerate() {
                for d in 0..dh {
                    for c in 0..TC {
                        let j = st * TC + c;
                        strip[d * TC + c] = if j < n { k[j * h + off + d] * s } else { 0.0 };
                    }
                }
            }
            for (st, strip) in vstrips.chunks_exact_mut(n * TC).enumerate() {
                for j in 0..n {
                    strip[j * TC..(j + 1) * TC]
                        .copy_from_slice(&v[j * h + off + st * TC..j * h + off + st * TC + TC]);
                }
            }
            // scores[i][j] = Σ_d (q[i][d]·s)·(k[j][d]·s), d in order
            gemm(&qs, dh, n, dh, &kstrips, &mut scores, np);
            for row in scores.chunks_exact_mut(np) {
                softmax(&mut row[..n]);
            }
            // ctx[i][d] = Σ_j p[i][j]·v[j][d], j in order
            gemm(&scores, np, n, n, &vstrips, &mut hctx, dh);
            for i in 0..n {
                ctx[i * h + off..i * h + off + dh].copy_from_slice(&hctx[i * dh..(i + 1) * dh]);
            }
        }
        let mut a = dense(&ctx, n, &l.o);
        for (y, r) in a.iter_mut().zip(x) {
            *y += r;
        }
        layer_norm(&mut a, h, &l.ln1_w, &l.ln1_b);
        let mut m = dense(&a, n, &l.i);
        for y in m.iter_mut() {
            *y = gelu(*y);
        }
        let mut o = dense(&m, n, &l.out);
        for (y, r) in o.iter_mut().zip(&a) {
            *y += r;
        }
        layer_norm(&mut o, h, &l.ln2_w, &l.ln2_b);
        o
    }
}

/// `y[t][j] = (Σᵢ x[t][i]·w[i][j]) + b[j]` ([`gemm`], then `+ b[j]`).
#[inline(always)]
fn dense(x: &[f32], n: usize, d: &Dense) -> Vec<f32> {
    let mut y = vec![0f32; n * d.dout];
    gemm(x, d.din, n, d.din, &d.strips, &mut y, d.dout);
    for row in y.chunks_exact_mut(d.dout) {
        for (v, b) in row.iter_mut().zip(&d.bias) {
            *v += b;
        }
    }
    y
}

/// `y[t][s·TC + c] = Σᵢ x[t][i]·w[i][s·TC + c]` for rows t < n (row t of `x`
/// at `t·xs`, `din` values; of `y` at `t·ys`) and every strip `s` of
/// `strips` (`din` rows of `TC` values each): one accumulator per output
/// component, `acc = fma(x[t][i], w[i][j], acc)` for i = 0, 1, … in order,
/// from 0 (fused multiply-add rounds once; IEEE 754 defines its result, so
/// every processor and the software fallback agree). Tiles of `TR` rows ×
/// `TC` columns keep their accumulators in registers; the tile shape never
/// changes a sum (a short last tile repeats its last row and drops it).
#[inline(always)]
fn gemm(x: &[f32], xs: usize, n: usize, din: usize, strips: &[f32], y: &mut [f32], ys: usize) {
    assert!(n == 0 || (n - 1) * xs + din <= x.len(), "gemm: input shape");
    assert!(strips.len().is_multiple_of(din * TC), "gemm: strips");
    let nstrips = strips.len() / (din * TC);
    assert!(
        n == 0 || (n - 1) * ys + nstrips * TC <= y.len(),
        "gemm: output shape"
    );
    for (s, strip) in strips.chunks_exact(din * TC).enumerate() {
        let mut t0 = 0;
        while t0 < n {
            let tr = TR.min(n - t0);
            let xr: [&[f32]; TR] = std::array::from_fn(|r| {
                let t = t0 + r.min(tr - 1);
                &x[t * xs..t * xs + din]
            });
            let acc = tile(&xr, strip);
            for (r, a) in acc.iter().enumerate().take(tr) {
                let o = (t0 + r) * ys + s * TC;
                y[o..o + TC].copy_from_slice(a);
            }
            t0 += tr;
        }
    }
}

/// One tile of [`gemm`]: `acc[r][c] = fma(x[r][i], w[i][c], acc[r][c])` for
/// i = 0, 1, … in order, from 0.
#[inline(always)]
fn tile(xr: &[&[f32]; TR], strip: &[f32]) -> [[f32; TC]; TR] {
    let din = xr[0].len();
    assert!(
        xr.iter().all(|r| r.len() == din) && strip.len() == din * TC,
        "tile: shapes"
    );
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is part of the aarch64 baseline; shapes checked
        unsafe { simd::tile_neon(xr, strip) }
    }
    #[cfg(target_arch = "x86_64")]
    {
        if simd::avx2_fma() {
            // SAFETY: the features were detected on this processor; shapes checked
            unsafe { simd::tile_avx2(xr, strip) }
        } else {
            tile_scalar(xr, strip)
        }
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    tile_scalar(xr, strip)
}

#[allow(dead_code, clippy::needless_range_loop)]
fn tile_scalar(xr: &[&[f32]; TR], strip: &[f32]) -> [[f32; TC]; TR] {
    let mut acc = [[0f32; TC]; TR];
    for (i, w) in strip.chunks_exact(TC).enumerate() {
        for r in 0..TR {
            let xv = xr[r][i];
            for c in 0..TC {
                acc[r][c] = xv.mul_add(w[c], acc[r][c]);
            }
        }
    }
    acc
}

// the kernels index lanes and rows by number, as the tile is drawn
#[allow(clippy::needless_range_loop)]
mod simd {
    use super::{TC, TR};

    #[cfg(target_arch = "x86_64")]
    pub fn avx2_fma() -> bool {
        std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma")
    }

    #[cfg(target_arch = "aarch64")]
    pub unsafe fn tile_neon(xr: &[&[f32]; TR], strip: &[f32]) -> [[f32; TC]; TR] {
        use std::arch::aarch64::*;
        const V: usize = TC / 4;
        let mut acc = [[vdupq_n_f32(0.0); V]; TR];
        let din = xr[0].len();
        let wp = strip.as_ptr();
        for i in 0..din {
            let w: [float32x4_t; V] = std::array::from_fn(|k| vld1q_f32(wp.add(i * TC + 4 * k)));
            for r in 0..TR {
                let xv = vdupq_n_f32(*xr[r].get_unchecked(i));
                for k in 0..V {
                    acc[r][k] = vfmaq_f32(acc[r][k], xv, w[k]);
                }
            }
        }
        let mut out = [[0f32; TC]; TR];
        for r in 0..TR {
            for k in 0..V {
                vst1q_f32(out[r].as_mut_ptr().add(4 * k), acc[r][k]);
            }
        }
        out
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,fma")]
    pub unsafe fn tile_avx2(xr: &[&[f32]; TR], strip: &[f32]) -> [[f32; TC]; TR] {
        use std::arch::x86_64::*;
        const V: usize = TC / 8;
        let mut acc = [[_mm256_setzero_ps(); V]; TR];
        let din = xr[0].len();
        let wp = strip.as_ptr();
        for i in 0..din {
            let w: [__m256; V] = std::array::from_fn(|k| _mm256_loadu_ps(wp.add(i * TC + 8 * k)));
            for r in 0..TR {
                let xv = _mm256_set1_ps(*xr[r].get_unchecked(i));
                for k in 0..V {
                    acc[r][k] = _mm256_fmadd_ps(xv, w[k], acc[r][k]);
                }
            }
        }
        let mut out = [[0f32; TC]; TR];
        for r in 0..TR {
            for k in 0..V {
                _mm256_storeu_ps(out[r].as_mut_ptr().add(8 * k), acc[r][k]);
            }
        }
        out
    }
}

/// ReduceMean, Sub, Pow 2, ReduceMean, Add ε, Sqrt, Div, Mul w, Add b; the
/// means are sums in index order divided by the width.
#[inline(never)]
fn layer_norm(x: &mut [f32], h: usize, w: &[f32], b: &[f32]) {
    let hf = h as f32;
    for row in x.chunks_exact_mut(h) {
        let mut sum = 0f32;
        for v in row.iter() {
            sum += v;
        }
        let mean = sum / hf;
        let mut sq = 0f32;
        for v in row.iter() {
            let d = v - mean;
            sq += d * d;
        }
        let sd = (sq / hf + LN_EPS).sqrt();
        for ((v, wv), bv) in row.iter_mut().zip(w).zip(b) {
            *v = (*v - mean) / sd * wv + bv;
        }
    }
}

/// max, `exp(x − max)` ([`exp64`] rounded to binary32), sum in order, divide.
/// `x − max ≤ 0`; below −104 `exp64` rounds to binary32 0, so the argument
/// is clamped there and the loop has no branch ([`exp64_neg`]).
#[inline(never)]
fn softmax(s: &mut [f32]) {
    let mut m = f32::NEG_INFINITY;
    for v in s.iter() {
        if *v > m {
            m = *v;
        }
    }
    for v in s.iter_mut() {
        *v = exp64_neg(f64::from(*v - m)) as f32;
    }
    let mut sum = 0f32;
    for v in s.iter() {
        sum += v;
    }
    for v in s.iter_mut() {
        *v /= sum;
    }
}

/// [`exp64`] for `x ∈ [−104, 0]` (smaller `x` give the value at −104,
/// whose binary32 rounding is 0, as `exp64`'s): the same operations without
/// branches (the exponent from the rounding sum's bits).
#[inline(always)]
fn exp64_neg(x: f64) -> f64 {
    const M: f64 = 6_755_399_441_055_744.0;
    let x = x.max(-104.0);
    let t = x * std::f64::consts::LOG2_E + M;
    let k = t - M; // round_even(x·log₂e)
    let ki = (t.to_bits() as i64).wrapping_sub(M.to_bits() as i64); // the same k as an integer
    let r = (x - k * LN2_HI) - k * LN2_LO;
    let f = &INV_FACT;
    // the Horner steps of `exp64`, written out
    let mut p = f[13];
    p = p * r + f[12];
    p = p * r + f[11];
    p = p * r + f[10];
    p = p * r + f[9];
    p = p * r + f[8];
    p = p * r + f[7];
    p = p * r + f[6];
    p = p * r + f[5];
    p = p * r + f[4];
    p = p * r + f[3];
    p = p * r + f[2];
    p = p * r + f[1];
    p = p * r + f[0];
    p * f64::from_bits(((ki + 1023) as u64) << 52)
}

/// The graph's GELU: `(x · (erf(x / 1.4142135) + 1)) · 0.5` in binary32,
/// `erf` by [`erf64`] rounded to binary32.
#[inline(never)]
fn gelu(x: f32) -> f32 {
    let e = erf64(f64::from(x / std::f32::consts::SQRT_2)) as f32;
    (x * (e + 1.0)) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The SIMD tile (NEON / AVX2+FMA) and the scalar `f32::mul_add` tile
    /// give the same bits: the scalar path runs on no CI runner otherwise.
    #[test]
    fn simd_tile_equals_scalar_tile() {
        let din = 37;
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            ((seed >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
        };
        let rows: Vec<Vec<f32>> = (0..TR)
            .map(|_| (0..din).map(|_| next()).collect())
            .collect();
        let strip: Vec<f32> = (0..din * TC).map(|_| next()).collect();
        let xr: [&[f32]; TR] = std::array::from_fn(|r| rows[r].as_slice());
        let a = tile(&xr, &strip);
        let b = tile_scalar(&xr, &strip);
        for r in 0..TR {
            for c in 0..TC {
                assert_eq!(a[r][c].to_bits(), b[r][c].to_bits(), "row {r} col {c}");
            }
        }
    }

    #[test]
    fn exp64_is_accurate() {
        for &x in &[-50.0, -10.5, -1.0, -1e-3, 0.0, 1e-3, 0.5, 1.0, 3.7, 20.0] {
            let rel = (exp64(x) - f64::exp(x)).abs() / f64::exp(x);
            assert!(rel < 4e-16, "exp64({x}): rel {rel}");
        }
        assert_eq!(exp64(-800.0), 0.0);
    }

    #[test]
    fn erf64_is_accurate() {
        // erf values to 16–17 digits; binary32 needs ~1e-8
        let known = [
            (0.1, 0.112_462_916_018_284_89),
            (0.5, 0.520_499_877_813_046_5),
            (1.0, 0.842_700_792_949_714_9),
            (2.0, 0.995_322_265_018_952_7),
            (3.0, 0.999_977_909_503_001_4),
            (4.5, 0.999_999_999_803_383_9),
        ];
        for (x, want) in known {
            assert!((erf64(x) - want).abs() < 1e-15, "erf64({x}) = {}", erf64(x));
            assert!((erf64(-x) + want).abs() < 1e-15);
        }
        assert_eq!(erf64(7.0), 1.0);
        let mut x = -6.5;
        while x < 6.5 {
            let d = (erf64(x) - erf64_series(x)).abs();
            assert!(d < 1e-14, "erf64({x}) table vs series: {d}");
            x += 0.001_23;
        }
    }
}
