class DataModel:
    def update(self, obj):
        val = obj.setting
        obj.config = "new_value"
        result = obj.data.value
