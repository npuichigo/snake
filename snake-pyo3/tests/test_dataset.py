import pytest
import snakedata


class TestDataset:
    @pytest.mark.asyncio
    async def test_range(self):
        dataset = snakedata.Dataset.range(0, 3)
        assert [x async for x in dataset] == [0, 1, 2]

    @pytest.mark.asyncio
    async def test_map(self):
        dataset = snakedata.Dataset.range(0, 3).map(lambda x: x * 2)
        assert [x async for x in dataset] == [0, 2, 4]
