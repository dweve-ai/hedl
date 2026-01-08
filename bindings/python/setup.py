"""
Setup script for HEDL Python bindings.
"""

from setuptools import setup, find_packages
from pathlib import Path

# Read README
readme_path = Path(__file__).parent / "README.md"
long_description = readme_path.read_text() if readme_path.exists() else ""

setup(
    name="hedl",
    version="1.0.0",
    author="Dweve",
    author_email="contact@dweve.com",
    description="HEDL (Hierarchical Entity Data Language) - Token-efficient data format for LLMs",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/dweve-ai/hedl",
    packages=find_packages(),
    package_data={
        "hedl": ["py.typed", "*.pyi"],
    },
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Text Processing :: Markup",
        "Typing :: Typed",
    ],
    python_requires=">=3.8",
    keywords="hedl data format llm context optimization json yaml csv parquet",
    project_urls={
        "Documentation": "https://github.com/dweve-ai/hedl/blob/main/docs/",
        "Source": "https://github.com/dweve-ai/hedl",
        "Issues": "https://github.com/dweve-ai/hedl/issues",
    },
    zip_safe=False,
)
