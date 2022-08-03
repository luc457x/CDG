from setuptools import setup, find_packages

setup(
    name='cdg',
    version='0.0.1',
    scripts=['cdg.py'],
    install_requires=[
        'matplotlib==3.5.2',
        'numpy==1.22.4',
        'pandas==1.4.2',
        'pandas_datareader==0.10.0',
        'pycoingecko==2.2.0',
        'python_dateutil==2.8.2',
        'requests_cache==0.9.5',
        'seaborn==0.11.2'
    ]
)
