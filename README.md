Vigil
=====

[![Build Status](https://travis-ci.org/valeriansaliou/vigil.svg?branch=master)](https://travis-ci.org/valeriansaliou/vigil)

**Microservices Status Page. Monitors a distributed infrastructure and sends alerts to Slack.**

Vigil is an open-source Status Page you can host on your infrastructure, used to monitor all your servers and apps, and visible to your users (on a domain of your choice, eg. `status.example.com`).

It is useful in microservices contexts to monitor both apps and backends. If a node goes down in your infrastructure, you receive a status change notification in a Slack channel.

**👉 See a live demo of Vigil on [Crisp Status Page](https://status.crisp.chat) and [Enrich Status Page](https://status.enrichdata.com).**

![Vigil](https://valeriansaliou.github.io/vigil/images/vigil.png)

## Who uses it?

<table>
<tr>
<td align="center"><a href="https://crisp.chat/"><img src="https://valeriansaliou.github.io/vigil/images/crisp-icon.png" height="64" /></a></td>
<td align="center"><a href="https://enrichdata.com/"><img src="https://valeriansaliou.github.io/vigil/images/enrich-icon.png" height="64" /></a></td>
</tr>
<tr>
<td align="center">Crisp</td>
<td align="center">Enrich</td>
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

Make sure to pick the correct server architecture (eg. Intel 32 bits).

**Install from Cargo:**

If you prefer managing `vigil` via Rust's Cargo, install it directly via `cargo install`:

```bash
cargo install vigil-server
```

Ensure that your `$PATH` is properly configured to source the Crates binaries, and then run Vigil using the `vigil` command.

**Install from source:**

The last option is to pull the source code from Git and compile Vigil via `cargo`:

```bash
cargo build --release
```

You can find the built binaries in the `./target/release` directory.

_Install the `libssl-dev` (ie. OpenSSL headers) before you compile Vigil. SSL dependencies are required for the HTTPS probes and email notifications._

**Install from Docker Hub:**

You might find it convenient to run Vigil via Docker. You can find the pre-built Vigil image on Docker Hub as [valeriansaliou/vigil](https://hub.docker.com/r/valeriansaliou/vigil/).

First, pull the `valeriansaliou/vigil` image:

```bash
docker pull valeriansaliou/vigil:v1.3.0
```

Then, seed it a configuration file and run it (replace `/path/to/your/vigil/config.cfg` with the path to your configuration file):

```bash
docker run -p 8080:8080 -v /path/to/your/vigil/config.cfg:/etc/vigil.cfg valeriansaliou/vigil:v1.3.0
```

In the configuration file, ensure that:

* `server.inet` is set to `0.0.0.0:8080` (this lets Vigil be reached from outside the container)
* `assets.path` is set to `./res/assets/` (this refers to an internal path in the container, as the assets are contained there)

Vigil will be reachable from `http://localhost:8080`.

### Configuration

Use the sample [config.cfg](https://github.com/valeriansaliou/vigil/blob/master/config.cfg) configuration file and adjust it to your own environment.

**Available configuration options are commented below, with allowed values:**

**[server]**

* `log_level` (type: _string_, allowed: `debug`, `info`, `warn`, `error`, default: `warn`) — Verbosity of logging, set it to `error` in production
* `inet` (type: _string_, allowed: IPv4 / IPv6 + port, default: `[::1]:8080`) — Host and TCP port the Vigil public status page should listen on
* `workers` (type: _integer_, allowed: any number, default: `4`) — Number of workers for the Vigil public status page to run on
* `reporter_token` (type: _string_, allowed: secret token, default: no default) — Reporter secret token (ie. secret password)

**[assets]**

* `path` (type: _string_, allowed: UNIX path, default: `./res/assets/`) — Path to Vigil assets directory

**[branding]**

* `page_title` (type: _string_, allowed: any string, default: `Status Page`) — Status page title
* `page_url` (type: _string_, allowed: URL, no default) — Status page URL
* `company_name` (type: _string_, allowed: any string, no default) — Company name (ie. your company)
* `icon_color` (type: _string_, allowed: hexadecimal color code, no default) — Icon color (ie. your icon background color)
* `icon_url` (type: _string_, allowed: URL, no default) — Icon URL, the icon should be your squared logo, used as status page favicon (PNG format recommended)
* `logo_color` (type: _string_, allowed: hexadecimal color code, no default) — Logo color (ie. your logo primary color)
* `logo_url` (type: _string_, allowed: URL, no default) — Logo URL, the logo should be your full-width logo, used as status page header logo (SVG format recommended)
* `website_url` (type: _string_, allowed: URL, no default) — Website URL to be used in status page header
* `support_url` (type: _string_, allowed: URL, no default) — Support URL to be used in status page header (ie. where users can contact you if something is wrong)
* `custom_html` (type: _string_, allowed: HTML, default: empty) — Custom HTML to include in status page `head` (optional)

**[metrics]**

* `poll_interval` (type: _integer_, allowed: seconds, default: `120`) — Interval for which to probe nodes in `poll` mode
* `poll_retry` (type: _integer_, allowed: seconds, default: `2`) — Interval after which to try probe for a second time nodes in `poll` mode (only when the first check fails)
* `poll_http_status_healthy_above` (type: _integer_, allowed: HTTP status code, default: `200`) — HTTP status above which `poll` checks to HTTP replicas reports as `healthy`
* `poll_http_status_healthy_below` (type: _integer_, allowed: HTTP status code, default: `400`) — HTTP status under which `poll` checks to HTTP replicas reports as `healthy`
* `poll_delay_dead` (type: _integer_, allowed: seconds, default: `30`) — Delay after which a node in `poll` mode is to be considered `dead` (ie. check response delay)
* `poll_delay_sick` (type: _integer_, allowed: seconds, default: `10`) — Delay after which a node in `poll` mode is to be considered `sick` (ie. check response delay)
* `push_delay_dead` (type: _integer_, allowed: seconds, default: `20`) — Delay after which a node in `push` mode is to be considered `dead` (ie. time after which the node did not report)
* `push_system_cpu_sick_above` (type: _float_, allowed: system CPU loads, default: `0.90`) — System load indice for CPU above which to consider a node in `push` mode `sick` (ie. UNIX system load)
* `push_system_ram_sick_above` (type: _float_, allowed: system RAM loads, default: `0.90`) — System load indice for RAM above which to consider a node in `push` mode `sick` (ie. percent RAM used)

**[plugins]**

**[plugins.rabbitmq]**

* `api_url` (type: _string_, allowed: URL, no default) — RabbitMQ API URL (ie. `http://127.0.0.1:15672`)
* `auth_username` (type: _string_, allowed: username, no default) — RabbitMQ API authentication username
* `auth_password` (type: _string_, allowed: password, no default) — RabbitMQ API authentication password
* `virtualhost` (type: _string_, allowed: virtual host, no default) — RabbitMQ virtual host hosting the queues to be monitored
* `queue_ready_healthy_below` (type: _integer_, allowed: any number, no default) — Maximum number of payloads in RabbitMQ queue with status `ready` to consider node `healthy`.
* `queue_nack_healthy_below` (type: _integer_, allowed: any number, no default) — Maximum number of payloads in RabbitMQ queue with status `nack` to consider node `healthy`.

**[notify]**

**[notify.email]**

* `to` (type: _string_, allowed: email address, no default) — Email address to which to send emails
* `from` (type: _string_, allowed: email address, no default) — Email address from which to send emails
* `smtp_host` (type: _string_, allowed: hostname, IPv4, IPv6, default: `localhost`) — SMTP host to connect to
* `smtp_port` (type: _integer_, allowed: TCP port, default: `587`) — SMTP TCP port to connect to
* `smtp_username` (type: _string_, allowed: any string, no default) — SMTP username to use for authentication (if any)
* `smtp_password` (type: _string_, allowed: any string, no default) — SMTP password to use for authentication (if any)
* `smtp_encrypt` (type: _boolean_, allowed: `true`, `false`, default: `true`) — Whether to encrypt SMTP connection with `STARTTLS` or not

**[notify.slack]**

* `hook_url` (type: _string_, allowed: URL, no default) — Slack hook URL (ie. `https://hooks.slack.com/[..]`)

**[probe]**

**[[probe.service]]**

* `id` (type: _string_, allowed: any unique lowercase string, no default) — Unique identifier of the probed service (not visible on the status page)
* `label` (type: _string_, allowed: any string, no default) — Name of the probed service (visible on the status page)

**[[probe.service.node]]**

* `id` (type: _string_, allowed: any unique lowercase string, no default) — Unique identifier of the probed service node (not visible on the status page)
* `label` (type: _string_, allowed: any string, no default) — Name of the probed service node (visible on the status page)
* `mode` (type: _string_, allowed: `poll`, `push`, no default) — Probe mode for this node (ie. `poll` is direct HTTP or TCP poll to the URLs set in `replicas`, while `push` is for Vigil Reporter nodes)
* `replicas` (type: _array[string]_, allowed: TCP or HTTP URLs, default: empty) — Node replica URLs to be probed (only used if `mode` is `poll`)
* `http_body_healthy_match` (type: _string_, allowed: regular expressions, no default) — HTTP response body for which to report node replica as `healthy` (if the body does not match, the replica will be reported as `dead`, even if the status code check passes; the check uses a `GET` rather than the usual `HEAD` if this option is set)
* `rabbitmq_queue` (type: _string_, allowed: RabbitMQ queue names, no default) — RabbitMQ queue associated to node, which to check against for pending payloads via RabbitMQ API (this helps monitor unacked payloads accumulating in the queue)

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

## What Slack alerts look like?

When a monitored backend or app goes down in your infrastructure, Vigil can let you know by Slack (or Email):

![Vigil alert in Slack](https://valeriansaliou.github.io/vigil/images/alert-slack.png)

You can also get nice realtime `down` and `up` alerts on your eg. iPhone and Apple Watch:

<p align="center">
  <img src="https://valeriansaliou.github.io/vigil/images/alert-slack-iphone.png" height="400" alt="Vigil down alert on iPhone" />
  <img src="https://valeriansaliou.github.io/vigil/images/alert-slack-watch.jpg" height="400" alt="Vigil up alert on Apple Watch" />
<p>

## How can I integrate Vigil Reporter in my code?

Vigil Reporter is used to actively submit health information to Vigil from your apps. Apps are best monitored via application probes, which are able to report detailed system information such as CPU and RAM load. This lets Vigil show if an application host system is under high load.

### Vigil Reporter Libraries

* **NodeJS**: **[node-vigil-reporter](https://www.npmjs.com/package/vigil-reporter)**
* **Golang**: **[go-vigil-reporter](https://github.com/valeriansaliou/go-vigil-reporter)**
* **Rust**: **[rs-vigil-reporter](https://crates.io/crates/vigil-reporter)**

👉 Cannot find the library for your programming language? Build your own and be referenced here! ([contact me](https://valeriansaliou.name/))

### Manual reporting

In case you need to manually report node metrics to the Vigil endpoint, use the following HTTP configuration (adjust it to yours):

**Endpoint URL:**

`HTTP POST https://status.example.com/reporter/<probe_id>/<node_id>/`

Where:

* `node_id`: The parent node of the reporting replica
* `probe_id`: The parent probe of the node

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `reporter_token`.

**Request data:**

Adjust the request data to your replica context and send it as `HTTP POST`:

```json
{
  "replica": "<replica_id>",
  "interval": 30,

  "load": {
    "cpu": 0.30,
    "ram": 0.80
  }
}
```

Where:

* `replica`: The replica unique identifier (eg. the server LAN IP)
* `interval`: The push interval (in seconds)
* `load.cpu`: The general CPU load, from `0.00` to `1.00` (can be more than `1.00` if the CPU is overloaded)
* `load.ram`: The general RAM load, from `0.00` to `1.00`

## :fire: Report A Vulnerability

If you find a vulnerability in Vigil, you are more than welcome to report it directly to [@valeriansaliou](https://github.com/valeriansaliou) by sending an encrypted email to [valerian@valeriansaliou.name](mailto:valerian@valeriansaliou.name). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production servers running an unpatched Vigil server.

**:warning: You must encrypt your email using [@valeriansaliou](https://github.com/valeriansaliou) GPG public key: [:key:valeriansaliou.gpg.pub.asc](https://valeriansaliou.name/files/keys/valeriansaliou.gpg.pub.asc).**

**:gift: Based on the severity of the vulnerability, I may offer a $100 (US) bounty to whomever reported it.**
