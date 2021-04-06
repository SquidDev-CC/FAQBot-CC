import functools


def with_async_timer(stat):
    def create(func):
        @functools.wraps(func)
        async def wrapped(*args, **kwargs):
            # Obtaining new instance of timer every time
            # ensures thread safety and reentrancy.
            with stat.time():
                return await func(*args, **kwargs)

        return wrapped

    return create
