"""
Configuration for the bot.

Ideally this wouldn't be a global. However, we need to access it in a few method
decorators, and so this is the cleanest solution.
"""

from typing import Optional, List
import json
import logging
import os

LOG = logging.getLogger(__name__)

_token: Optional[str] = None
_guild_ids: Optional[List[int]] = None


def _load() -> None:
    global _token, _guild_ids

    if _token is not None:
        return

    if os.path.isfile("config.json"):
        with open('config.json', 'r') as file:
            config = json.load(file)

        _token = config['token']
        _guild_ids = config.get('guild_ids')
        LOG.info('Loaded config from config.json')

    else:
        with open('token', 'r') as file:
            _token = file.read().strip()
        LOG.info('Loaded config from token file')


def token() -> str:
    _load()
    if _token is None:
        raise ValueError("Config could not be loaded")
    return _token


def guild_ids() -> Optional[List[int]]:
    _load()
    return _guild_ids
