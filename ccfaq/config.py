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
_metrics_port: Optional[int] = None
_eval_server: str = "https://eval.tweaked.cc"


def _load() -> None:
    global _token, _guild_ids, _metrics_port, _eval_server

    if _token is not None:
        return

    if os.path.isfile("config.json"):
        with open('config.json', 'r') as file:
            config = json.load(file)

        _token = config['token']
        _guild_ids = config.get('guild_ids')
        _metrics_port = config.get('metrics_port')
        _eval_server = config.get('eval_server', _eval_server)
        LOG.info('Loaded config from config.json')

    else:
        with open('token', 'r') as file:
            _token = file.read().strip()
        LOG.info('Loaded config from token file')


def token() -> str:
    """Token to connect to Discord with."""
    _load()
    if _token is None:
        raise ValueError("Config could not be loaded")
    return _token


def guild_ids() -> Optional[List[int]]:
    """Restricted guild ids this bot registers commands under."""
    _load()
    return _guild_ids


def metrics_port() -> Optional[int]:
    """Port to expose metrics on."""
    _load()
    return _metrics_port


def eval_server() -> str:
    """The server to use for evaling code."""
    _load()
    return _eval_server
