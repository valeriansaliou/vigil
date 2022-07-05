Vigil Protocol
==============

# Vigil Reporter HTTP API

## 1️⃣ Report a replica

**Endpoint URL:**

`HTTP POST https://status.example.com/reporter/<probe_id>/<node_id>/`

Where:

* `node_id`: The parent node of the reporting replica
* `probe_id`: The parent probe of the node

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `reporter_token`.
* Set the `Content-Type` to `application/json; charset=utf-8`, and ensure you submit the request data as UTF-8.

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

## 2️⃣ Flush a replica

**Endpoint URL:**

`HTTP DELETE https://status.example.com/reporter/<probe_id>/<node_id>/<replica_id>/`

Where:

* `node_id`: The parent node of the reporting replica
* `probe_id`: The parent probe of the node
* `replica_id`: The replica unique identifier (eg. the server LAN IP)

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `reporter_token`.

---

# Vigil Manager HTTP API

## 1️⃣ List published announcements

**Endpoint URL:**

`HTTP GET https://status.example.com/manager/announcements/`

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `manager_token`.

## 2️⃣ Insert a new announcement

**Endpoint URL:**

`HTTP POST https://status.example.com/manager/announcement/`

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `manager_token`.
* Set the `Content-Type` to `application/json; charset=utf-8`, and ensure you submit the request data as UTF-8.

**Request data:**

Adjust the request data to your announcement and send it as `HTTP POST`:

```json
{
  "title": "<title>",
  "text": "<text>"
}
```

Where:

* `title`: The title for the announcement
* `text`: The description text for the announcement (can be multi-line)

## 3️⃣ Retract a published announcement

**Endpoint URL:**

`HTTP DELETE https://status.example.com/manager/announcement/<announcement_id>/`

Where:

* `announcement_id`: The announcement identifier to be removed

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `manager_token`.

## 4️⃣ List prober alerts

**Endpoint URL:**

`HTTP GET https://status.example.com/manager/prober/alerts/`

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `manager_token`.

## 5️⃣ Resolve ignore rules for prober alerts

**Endpoint URL:**

`HTTP GET https://status.example.com/manager/prober/alerts/ignored/`

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `manager_token`.

## 6️⃣ Update ignore rules for prober alerts

**Endpoint URL:**

`HTTP PUT https://status.example.com/manager/prober/alerts/ignored/`

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `manager_token`.
* Set the `Content-Type` to `application/json; charset=utf-8`, and ensure you submit the request data as UTF-8.

**Request data:**

Adjust the request data to your announcement and send it as `HTTP PUT`:

```json
{
  "reminders_seconds": 600
}
```

Where:

* `reminders_seconds`: The number of seconds during which downtime reminders should not be sent anymore (skipped)
