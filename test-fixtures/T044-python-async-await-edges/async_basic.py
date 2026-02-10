async def load_data():
    result = await fetch_data()
    await self.save_async()
