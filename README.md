# Home Automation - Tapo

[![CI][ci_badge]][ci]
[![license][license_badge]][license]

Reads the `device_usage` of multiple devices and sends the data through MQTT to be picked up by [home-automation-monitoring](https://github.com/mihai-dinculescu/home-automation-monitoring).
It also includes an API that can control the devices.

Actor System consisting of:

- Coordinator Actor - makes sure that everything is running as expected
- Device Actor - reads the device usage and sends it to the MQTT Actor
- MQTT Actor - published the data to the MQTT broker
- API Actor - REST API for turning devices on/off and getting their status

## Usage

Rename and update `settings.sample.yaml` to `settings.yaml`.

```bash
cargo run
```

## Docker

### linux/amd64 & linux/arm64

```bash
docker build -t home-automation-tapo .
docker run -d -p 80:80 home-automation-tapo
```

### linux/arm/v7

```bash
docker build -f Dockerfile-ARMv7.dockerfile -t home-automation-tapo .
docker run -d -p 80:80 home-automation-tapo
```

[ci_badge]: https://github.com/mihai-dinculescu/home-automation-tapo/workflows/CI/badge.svg?branch=main
[ci]: https://github.com/mihai-dinculescu/home-automation-tapo/actions
[license_badge]: https://img.shields.io/crates/l/home-automation-tapo.svg
[license]: https://github.com/mihai-dinculescu/home-automation-tapo/blob/main/LICENSE
