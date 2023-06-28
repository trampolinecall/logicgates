from . import ty

class Bit:
    def __init__(self):
        pass

    def type(self):
        return ty.Bit()

class ListProduct:
    def __init__(self, *fields):
        self.fields = fields

    def __getitem__(self, key):
        return self.fields[key]

    def type(self):
        return ty.ListProduct(*[field.type() for field in self.fields])

class DictProduct:
    def __init__(self, **fields):
        self.fields = fields

    def __getitem__(self, key):
        return self.fields[key]

    def type(self):
        return ty.DictProduct(**{name: field.type() for (name, field) in self.fields.items()})
