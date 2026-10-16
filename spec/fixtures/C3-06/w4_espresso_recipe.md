# Espresso

Espresso is brewed with nine bars of pressure for about 28 seconds.

## Recipe

- Dose: 20 g
- Yield: 40 g

## Equipment

The shop uses a two-group machine that is descaled every month according to [[maintenance]].

```python
def ratio(dose, yield_):
    return yield_ / dose  # 1:2 is the house standard
```

The house blend is defined in [[blends]].

## Agent tips

The agent thinks a longer preinfusion helps light roasts.
