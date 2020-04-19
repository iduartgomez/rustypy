# Release procedure

1. Ensure pypandoc & pandoc are installed:
    > $ apt-get install -y pandoc \
      $ pip install "pypandoc==1.4"
2. Ensure version numbers are aligned (
    `./src/librustypy/Cargo.toml`, 
    `.src/rustypy/__init__.py`, 
    `setup.py`
)
3. Build python package:
    > $ python setup.py sdist bdist wheel
4. Publish:
    > $ cargo publish \
    > $ twine -u *** -p *** upload dist/*
 