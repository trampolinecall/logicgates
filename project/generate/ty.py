from . import bundle

class Bit:
    def __init__(self):
        pass

    def __eq__(self, other):
        return isinstance(other, Bit)

    def __str__(self):
        return 'Bit'

    def make_bundle(self):
        return bundle.Bit()

    def size(self):
        return 1

class ListProduct:
    def __init__(self, *fields):
        self.fields = fields

    def __eq__(self, other):
        if isinstance(other, ListProduct):
            return self.fields == other.fields
        else:
            return False

    def __str__(self):
        return str(self.fields)

    def make_bundle(self):
        return bundle.ListProduct(*[field.make_bundle() for field in self.fields])

    def size(self):
        return sum([field.size() for field in self.fields])

class DictProduct:
    def __init__(self, **fields):
        self.fields = fields

    def __eq__(self, other):
        if isinstance(other, DictProduct):
            return self.fields == other.fields
        else:
            return False

    def __str__(self):
        return str(self.fields)

    def make_bundle(self):
        return bundle.DictProduct(**{name: ty.make_bundle() for (name, ty) in self.fields.items()})

    def size(self):
        return sum([ty.size() for ty in self.fields.values()])
