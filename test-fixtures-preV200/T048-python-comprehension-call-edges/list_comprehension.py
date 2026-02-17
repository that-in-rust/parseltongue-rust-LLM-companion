class Transformer:
    def transform(self, items):
        names = [process(x) for x in items]
        filtered = [validate(y) for y in data if check(y)]
