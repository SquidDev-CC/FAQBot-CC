"""Main entrypoint to the bot."""

import ccfaq
import ccfaq.log
import ccfaq.telemetry

if __name__ == '__main__':
    ccfaq.telemetry.configure()
    ccfaq.log.configure()
    ccfaq.run()
