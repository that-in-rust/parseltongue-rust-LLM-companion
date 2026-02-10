class Factory:
    def build(self):
        user = models.User()
        db = database.Connection()
