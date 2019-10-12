# -*- coding: utf-8 -*-
import os
import sys

from setuptools import find_packages, setup
from io import StringIO

if sys.version_info[0:2] < (3, 5):
    raise RuntimeError("Python version >= 3.5 required.")

path = os.path.abspath(os.path.dirname(__file__))


def get_rustypy_module():
    sys.path.append(os.path.join(path, 'src'))
    import rustypy
    return rustypy


rustypy = get_rustypy_module()


def generate_description():
    # get long description
    f = os.path.join(path, 'README.md')
    try:
        from pypandoc import convert
        long_description = '\n' + convert(f, 'rst')
    except ImportError:
        import logging
        logging.warning(
            "warning: pypandoc module not found, could not convert " +
            "Markdown to RST")
        long_description = '\n' + open(f, 'r').read()
    return long_description


def update_cargo_version():
    import re
    new_file = StringIO()
    rslib = os.path.join(path, 'src', 'rslib')
    old_ver = re.compile(r'^version = "(.*)"')
    ori_toml = os.path.join(rslib, 'Cargo.toml')
    new_toml = os.path.join(rslib, 'Cargo.temp.toml')
    with open(ori_toml, 'r') as f:
        for l in f:
            ver = re.match(old_ver, l)
            if ver:
                version = 'version = "{}"\n'.format(rustypy.__version__)
                new_file.write(version)
            else:
                new_file.write(l)
    f = open(new_toml, 'w')
    f.write(new_file.getvalue())
    f.close()
    os.remove(ori_toml)
    os.rename(new_toml, ori_toml)


setup(
    name="rustypy",
    version=rustypy.__version__,
    description='Automatic FFI generation for Python <-> Rust interfacing.',
    long_description=generate_description(),
    url='https://github.com/iduartgomez/rustypy',
    author='Ignacio Duart Gómez',
    author_email='iduartgomez@gmail.com',
    license='BSD 3-Clause',
    # See https://pypi.python.org/pypi?%3Aaction=list_classifiers
    classifiers=[
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Operating System :: POSIX',
        'Programming Language :: Python :: 3.5',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
        'Programming Language :: Rust',
        'Topic :: Software Development :: Code Generators'
    ],
    keywords='rust autogenerated FFI',
    package_data={'rslib': ['*/*']},
    packages=find_packages('src'),
    package_dir={'': 'src'},
    # setup_requires=['cffi'],
    # install_requires=['cffi'],
    entry_points={
        'console_scripts': [
            'rustypy=rustypy.scripts:cli',
        ],
    },
)
