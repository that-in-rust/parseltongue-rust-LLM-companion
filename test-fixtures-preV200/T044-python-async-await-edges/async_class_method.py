class Fetcher:
    async def load(self):
        data = await get_user()
        result = await process_data(data)
