"""Main entrypoint to the bot."""

import ccfaq
import ccfaq.log

if __name__ == '__main__':
    ccfaq.log.configure()
    ccfaq.run()
