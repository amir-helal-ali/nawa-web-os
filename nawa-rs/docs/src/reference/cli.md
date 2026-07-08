# CLI Commands

## nawa create

```
nawa create <NAME> [--template <T>] [--dir <D>]

Arguments:
  <NAME>  Project name (becomes directory name)

Options:
  -t, --template <T>  Template to use [default: blog]
  -d, --dir <D>       Target directory [default: current dir]
```

Templates: `blog`, `saas`, `shop`, `realtime`, `booking`, `portfolio`

## nawa dev

```
nawa dev [--addr <ADDR>] [--data-dir <DIR>]

Options:
  --addr <ADDR>      Address to bind [default: 0.0.0.0:8080]
  --data-dir <DIR>   Data directory [default: ./nawa-data]
```

Hot reload: watches `src/` and rebuilds on change.

## nawa build

```
nawa build [--release <BOOL>]

Options:
  --release <BOOL>   Build in release mode [default: true]
```

## nawa deploy

```
nawa deploy [--target <SSH>] [--remote-data-dir <DIR>]

Options:
  --target <SSH>             SSH target (e.g., user@host)
  --remote-data-dir <DIR>    Remote data directory [default: /var/lib/nawa]
```

4-step deployment: build → tarball → scp → ssh install.

## nawa benchmark

```
nawa benchmark [--ops <N>]

Options:
  -o, --ops <N>   Number of operations [default: 100000]
```

## nawa info

Shows version, components, and platform info.

## nawa templates

Lists all available templates.
