Vigil
=====

[![Build Status](https://travis-ci.org/valeriansaliou/vigil.svg?branch=master)](https://travis-ci.org/valeriansaliou/vigil)

**Microservices Status Page. Monitors a distributed infrastructure and sends alerts to Slack.**

Vigil is an open-source Status Page you can host on your infrastructure, used to monitor all your servers and apps, and visible to your users (on a domain of your choice, eg. `status.example.com`).

It is useful in microservices contexts to monitor both apps and backends. If a node goes down in your infrastructure, you receive a status change notification in a Slack channel.

**👉 See a live demo of Vigil on [Crisp Status Page](https://status.crisp.chat).**

**🚨 Vigil is currently Work In Progress (WIP). Stable version is coming soon (1 week ETA).**

![Vigil](https://valeriansaliou.github.io/vigil/images/vigil.png)

## Who uses it?

<table>
<tr>
<td align="center"><a href="https://crisp.chat/"><img src="https://valeriansaliou.github.io/vigil/images/crisp-icon.png" height="64" /></a></td>
</tr>
<tr>
<td align="center">Crisp</td>
</tr>
</table>

_👋 You use Vigil and you want to be listed there? [Contact me](https://valeriansaliou.name/)._

## Features

* **Monitors automatically your infrastructure services**
* **Notifies you when a service gets down** or gets back up (via a configured channel, eg. Slack or Email)
* **Generates a status page**, that you can host on your domain for your public users (eg. `https://status.example.com`)

## How does it work?

Vigil monitors all your infrastructure services. You first need to configure target services to be monitored, and then Vigil does the rest for you.

**There are two kinds of services Vigil can monitor:**

* **HTTP / TCP services**: Vigil frequently probe a HTTP or TCP target and checks for reachability
* **Application services**: Install the Vigil Reporter library eg. on your NodeJS app and get reports when your app gets down, as well as when the host server system is overloaded

It is recommended to configure Vigil or Vigil Reporter to send frequent probe checks, as to ensure you are quickly notified when a service gets down (thus to reduce unexpected downtime on your services).

## How to use it?

### Installation

**Install from releases:**

The best way to install Vigil is to pull the latest release from the [Vigil releases](https://github.com/valeriansaliou/vigil/releases) page.

Make sure to pick the correct server architecture (either Intel 32 bits, Intel 64 bits, or ARM).

**Install from Cargo:**

If you prefer managing `vigil` via Rust's Cargo, install it directly via `cargo install`:

```bash
cargo install vigil-server
```

Ensure that your `$PATH` is properly configured to source the Crates binaries, and then run Vigil using the `vigil` command.

**Install from sources:**

The last option is to pull the source code from Git and compile Vigil via `cargo`:

```bash
cargo build --release
```

You can find the built binaries in the `./target/release` directory.

### Configuration

Use the sample [config.cfg](https://github.com/valeriansaliou/vigil/blob/master/config.cfg) configuration file and adjust it to your own environment.

**Available configuration options are commented below, with allowed values:**

**[server]**

* `log_level` (type: _string_, allowed: `debug`, `info`, `warn`, `error`, default: `warn`) — Verbosity of logging, set it to `error` in production
* `inet` (type: _string_, allowed: IPv4 / IPv6 + port, default: `[::1]:8080`) — Host and TCP port the Vigil public status page should listen on

**[assets]**

* `path` (type: _string_, allowed: UNIX path, default: `./res/assets/`) — Path to Vigil assets directory

**[branding]**

* `page_title` (type: _string_, allowed: any string, default: `Status Page`) — Status page title
* `company_name` (type: _string_, allowed: any string, no default) — Company name (ie. your company)
* `icon_color` (type: _string_, allowed: any valid hexadecimal color code, no default) — Icon color (ie. your icon background color)
* `icon_url` (type: _string_, allowed: any valid URL, no default) — Icon URL, the icon should be your squared logo, used as status page favicon (PNG format recommended)
* `logo_color` (type: _string_, allowed: any valid hexadecimal color code, no default) — Logo color (ie. your logo primary color)
* `logo_url` (type: _string_, allowed: any valid URL, no default) — Logo URL, the logo should be your full-width logo, used as status page header logo (SVG format recommended)
* `website_url` (type: _string_, allowed: any valid URL, no default) — Website URL to be used in status page header
* `support_url` (type: _string_, allowed: any valid URL, no default) — Support URL to be used in status page header (ie. where users can contact you if something is wrong)
* `custom_html` (type: _string_, allowed: any valid HTML, default: empty) — Custom HTML to include in status page `head` (optional)

**[probe]**

**[[probe.service]]**

* `id` (type: _string_, allowed: any unique lowercase string, no default) — Unique identifier of the probed service (not visible on the status page)
* `label` (type: _string_, allowed: any string, no default) — Name of the probed service (visible on the status page)

**[[probe.service.node]]**

* `id` (type: _string_, allowed: any unique lowercase string, no default) — Unique identifier of the probed service node (not visible on the status page)
* `label` (type: _string_, allowed: any string, no default) — Name of the probed service node (visible on the status page)
* `mode` (type: _string_, allowed: `poll`, `push`, no default) — Probe mode for this node (ie. `poll` is direct HTTP or TCP poll to the URLs set in `replicas`, while `push` is for Vigil Reporter nodes)
* `replicas` (type: _array[string]_, allowed: any valid array of TCP or HTTP URLs, default: `[]`) — Node replica URLs to be probed (only used if `mode` is `poll`)

### Run Vigil

Vigil can be run as such:

`./vigil -c /path/to/config.cfg`

## Usage recommendations

**Consider the following recommendations when using Vigil:**

* **Vigil should be hosted on a safe, separate server.** This server should run on a different physical machine and network than your monitored infrastructure servers.
* **Make sure to whitelist the Vigil server public IP (both IPv4 and IPv6)** on your monitored HTTP services; this applies if you use a bot protection service that challenges bot IPs, eg. Distil Networks or Cloudflare. Vigil will see the HTTP service as down if a bot challenge is raised.

## What status variants look like?

Vigil has 3 status variants, either `healthy` (no issue ongoing), `sick` (services under high load) or `dead` (outage):

### Healthy status variant

![Status Healthy](https://valeriansaliou.github.io/vigil/images/status-healthy.png)

### Sick status variant

![Status Sick](https://valeriansaliou.github.io/vigil/images/status-sick.png)

### Dead status variant

![Status Dead](https://valeriansaliou.github.io/vigil/images/status-dead.png)

## How can I integrate Vigil Reporter in my code?

Vigil Reporter is used to actively submit health information to Vigil from your apps. Apps are best monitored via application probes, which are able to report detailed system information such as CPU and RAM load. This lets Vigil show if an application host system is under high load.

**📦 Vigil Reporter Libraries:**

* **NodeJS**: **[node-vigil-reporter](https://www.npmjs.com/package/vigil-reporter)**
* **Rust**: **[rs-vigil-reporter](https://crates.io/crates/vigil-reporter)**
* **Golang**: **[go-vigil-reporter](https://github.com/valeriansaliou/go-vigil-reporter)**

👉 Cannot find the library for your programming language? Build your own and be referenced here! ([contact me](https://valeriansaliou.name/))

## :fire: Report A Vulnerability

If you find a vulnerability in Vigil, you are more than welcome to report it directly to [@valeriansaliou](https://github.com/valeriansaliou) by sending an encrypted email to [valerian@valeriansaliou.name](mailto:valerian@valeriansaliou.name). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production servers running an unpatched Vigil server.

**:warning: You must encrypt your email using [@valeriansaliou](https://github.com/valeriansaliou) GPG public key: [:key:valeriansaliou.gpg.pub.asc](https://valeriansaliou.name/files/keys/valeriansaliou.gpg.pub.asc).**

**:gift: Based on the severity of the vulnerability, I may offer a $100 (US) bounty to whomever reported it.**
