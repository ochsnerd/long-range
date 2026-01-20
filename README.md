# Long Range Percolation

## Usage

```
$ nix run

In [1]: import testpkg

In [2]: a = testpkg.simulate(100, 0.3, 0.2, 1000, 42)

In [3]: a[0].average_size
```

Or download the appropriate build artefact for your system. Then, 

```
python -m pip install <foo>.whl
```
