
"""
Pandoc filter to add ../ prefix to all file paths in the document.

This filter processes:
- Image paths in markdown images
- Bibliography file paths in metadata
- CSL file paths in metadata
- Any other file references

Usage:
    pandoc input.md --filter pandoc/path_prefix_filter.py -o output.pdf
"""

import re
import os
from typing import Any, Union
import panflute as pf


def add_prefix_to_path(path: str, prefix: str = "../") -> str:
    """
    Add prefix to a path if it's not already absolute or a URL.
    
    Args:
        path: The original path
        prefix: The prefix to add (default: "../")
    
    Returns:
        The path with prefix added, or original path if it's absolute/URL
    """
    if not path:
        return path
    
    # Don't modify absolute paths or URLs
    if (path.startswith('/') or 
        path.startswith('http://') or 
        path.startswith('https://') or
        path.startswith('file://') or
        os.path.isabs(path)):
        return path
    
    # Don't add prefix if it's already there
    if path.startswith(prefix):
        return path
    
    return prefix + path


def process_metadata_value(value: Any, prefix: str = "../") -> Any:
    """
    Process panflute metadata values that might contain file paths.
    
    Args:
        value: The metadata value to process (panflute MetaValue)
        prefix: The prefix to add to paths
    
    Returns:
        Processed metadata value (panflute MetaValue)
    """
    # Handle MetaString
    if isinstance(value, pf.MetaString):
        return pf.MetaString(add_prefix_to_path(value.text, prefix))
    
    # Handle MetaInlines (inline text)
    elif isinstance(value, pf.MetaInlines):
        # Extract text from inlines
        text = pf.stringify(value)
        return pf.MetaString(add_prefix_to_path(text, prefix))
    
    # Handle MetaList (list of values)
    elif isinstance(value, pf.MetaList):
        processed_items = [process_metadata_value(item, prefix) for item in value.content]
        return pf.MetaList(*processed_items)
    
    # Handle MetaMap (dictionary)
    elif isinstance(value, pf.MetaMap):
        processed_dict = {k: process_metadata_value(v, prefix) for k, v in value.content.items()}
        return pf.MetaMap(**processed_dict)
    
    # Return unchanged for other types
    else:
        return value


def action(elem: pf.Element, doc: pf.Doc) -> Union[pf.Element, None]:
    """
    Panflute action function to process each element.
    
    Args:
        elem: The element to process
        doc: The document
    
    Returns:
        Modified element or None
    """
    # Process images
    if isinstance(elem, pf.Image):
        elem.url = add_prefix_to_path(elem.url)
        return elem
    
    # Process links that might reference local files
    if isinstance(elem, pf.Link):
        # Only modify if it looks like a file path (has extension)
        if '.' in elem.url and not elem.url.startswith('http'):
            elem.url = add_prefix_to_path(elem.url)
        return elem
    
    return None


def prepare(doc: pf.Doc) -> None:
    """
    Prepare function to process document metadata before elements.
    
    Args:
        doc: The document
    """
    # Process bibliography paths
    if 'bibliography' in doc.metadata:
        doc.metadata['bibliography'] = process_metadata_value(doc.metadata['bibliography'])
    
    # Process CSL path
    if 'csl' in doc.metadata:
        doc.metadata['csl'] = process_metadata_value(doc.metadata['csl'])
    
    # Process other common metadata paths
    path_metadata_keys = [
        'reference-doc',
        'reference-docx', 
        'template',
        'include-before-body',
        'include-after-body',
        'include-in-header',
        'css',
        'data-dir',
        'extract-media',
        'resource-path'
    ]
    
    for key in path_metadata_keys:
        if key in doc.metadata:
            doc.metadata[key] = process_metadata_value(doc.metadata[key])


def finalize(doc: pf.Doc) -> None:
    """
    Finalize function called after all elements are processed.
    
    Args:
        doc: The document
    """
    pass


def main(doc: pf.Doc | None = None):
    """
    Main function for the filter.
    
    Args:
        doc: The document (provided by panflute)
    
    Returns:
        Processed document
    """
    return pf.run_filter(
        action,
        prepare=prepare,
        finalize=finalize,
        doc=doc
    )


if __name__ == '__main__':
    main()

