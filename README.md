# Home Automation - Tapo

Reads the `device_usage` of multiple devices and sent the data through MQTT to be picked up by [home-automation-monitoring](https://github.com/mihai-dinculescu/home-automation-monitoring).

## Usage

Rename and update `settings.sample.yaml` to `settings.yaml`.

```bash
cargo run
```

## Docker

### linux/amd64 & linux/arm64

```bash
docker build -t home-automation-tapo .
docker run -d home-automation-tapo
```

### linux/arm/v7

```bash
docker build -f Dockerfile-ARMv7.dockerfile -t home-automation-tapo .
docker run -d home-automation-tapo
```
