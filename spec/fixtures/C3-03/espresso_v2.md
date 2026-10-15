# Espresso

Espresso is brewed with nine bars of pressure for about 28 seconds; extraction time and pressure decide the house blend recipe.

## Recipe

- Dose: 18 g
- Yield: 36 g
- Time: 26 to 30 seconds
  - Under 24 seconds tastes sour.
  - Over 32 seconds tastes bitter.

## Equipment

The shop uses a two-group machine that is descaled every month according to [[maintenance]].

```python
def ratio(dose, yield_):
    return yield_ / dose  # 1:2 is the house standard
```

The house blend is defined in [[blends]].
