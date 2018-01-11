Vigil
=====

[![Build Status](https://travis-ci.org/valeriansaliou/vigil.svg?branch=master)](https://travis-ci.org/valeriansaliou/vigil)

**Microservices Status Page. Monitors a distributed infrastructure and sends alerts to Slack.**

Vigil is an open-source Status Page you can host on your infrastructure, used to monitor all your servers and apps, and visible to your users (on a domain of your choice, eg. `status.example.com`).

It is useful in microservices contexts to monitor both apps and backends. If a node goes down in your infrastructure, you receive a status change notification in a Slack channel.

**👉 See a live demo of Vigil on [Crisp Status Page](https://status.crisp.chat).**

**🚨 Vigil is currently Work In Progress (WIP). Stable version is coming soon (2 months ETA).**

![Vigil](https://valeriansaliou.github.io/vigil/images/vigil.png)

## Who uses it?

<table>
<tr>
<td align="center"><a href="https://crisp.chat/"><img src="https://valeriansaliou.github.io/vigil/images/crisp.png" height="64" /></a></td>
</tr>
<tr>
<td align="center">Crisp</td>
</tr>
</table>

_👋 You use Vigil and you want to be listed there? [Contact me](https://valeriansaliou.name/)._

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
