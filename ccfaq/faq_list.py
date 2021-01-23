"""Loads FAQS from the appropriate directory"""
from typing import Optional, List
import dataclasses
import logging
import pathlib
import re

import yaml

LOG = logging.getLogger(__name__)


_front_matter = re.compile("---\n(.*?)\n---\n(.*)$", re.MULTILINE | re.DOTALL)


@dataclasses.dataclass
class FAQ:
    """An FAQ"""
    name: str
    contents: str

    title: str
    search: str


def load_file(path: pathlib.Path) -> Optional[FAQ]:
    """Load a single FAQ from a file."""
    with open(path) as handle:
        contents = handle.read()

    frontmatter = _front_matter.match(contents)
    if not frontmatter:
        LOG.error("No frontmatter for %s", path.name)
        return None

    try:
        front = yaml.load(frontmatter[1], Loader=yaml.SafeLoader)
    except yaml.YAMLError:
        LOG.exception("Cannot parse frontmatter %s", path)

    if path.name == "example.md":
        LOG.info("Skipping example file")
        return None

    front['name'] = path.name[:-3].replace('.', '')
    front['contents'] = frontmatter[2].strip()

    return FAQ(**front)


def load() -> List[FAQ]:
    """Load all FAQs"""
    faqs: List[FAQ] = []
    for file in pathlib.Path('faqs').glob('*.md'):
        faq = load_file(file)
        if faq is not None:
            faqs.append(faq)

    LOG.info("Loaded %d FAQs", len(faqs))
    return faqs
