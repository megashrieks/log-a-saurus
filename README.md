# Log-a-saurus
Log ingestion server optimised for high throughput and low latency

## Running

build the image using `docker compose build`

run the image using `docker compose up`

stop the image using `docker compose down`

## Log Structure
```
{
	"level": string,
	"message": string,
    "resourceId": string,
	"timestamp": "<timestamp>", // example: "2023-09-15T08:00:00Z"
	"traceId": string,
    "spanId": string,
    "commit": string,
    "metadata": {
        "parentResourceId": string
    }
}
```

## Ingestion
Ingest the data to the server by sending a post request with the log as the body of the request at port 3000

## Querying

Querying cli / web ui has not been implemented yet, querying has to be done over a HTTP call over the url `/query`

Querying of each column and combining multiple columns is supported along with operators

Operators supported:
 - LIKE and EQUALS for text based columns
 - EQUALS and BETWEEN for timebased interfaces

Pagination should be done through top and offset properties

Example Query Body
```json
{
    "level": {"equals": "error"},
    "message": {"like": "failed"},
    "pagination": { "top": 100, "offset": 0}
}
```
Complete Example
```json
{
    "level": {"equals": "error"},
    "message": {"like": "Failed%"},
    "resourceId": {"equals": "server-1234"},
    "commit": {"like": "%534%"},
    "traceId": {"like": "%xyz%"},
    "spanId": {"equals": "span-456"},
    "metadata": {
        "parentResourceId": {"equals":"server-0987"}
    },
    "timestamp": {
        "from": "2020-12-10T00:00:00.000Z",
        "to": "2025-12-10T00:00:00.000Z"
    },
    "pagination": { "top": 100, "offset": 0}
}
```

## cURL
```
Querying:

curl --location 'localhost:3000/query' \
--header 'Content-Type: application/json' \
--data '{
    "level": {"equals": "error"},
    "message": {"like": "Failed%"},
    "resourceId": {"equals": "server-1234"},
    "commit": {"like": "%534%"},
    "traceId": {"like": "%xyz%"},
    "spanId": {"equals": "span-456"},
    "metadata": {
        "parentResourceId": {"like":"server%"}
    },
    "timestamp": {
        "from": "2020-12-10T00:00:00.000Z",
        "to": "2025-12-10T00:00:00.000Z"
    },
    "pagination": { "top": 100, "offset": 10000}
}'

Insertion:

curl --location 'localhost:3000/' \
--header 'Content-Type: application/json' \
--data '{
    "level": "error",
    "message": "Failed to connect to DB",
    "resourceId": "server-1234",
    "timestamp": "2023-09-15T08:00:00Z",
    "traceId": "abc-xyz-123",
    "spanId": "span-456",
    "commit": "5e5342f",
    "metadata": {
        "parentResourceId": "server-0987",
        "asd": "Asd"
    }
}'

```

## Load Test
Basic artillery script is located in the root of the repo, to run
```
npm i -g artillery
artillery run artillery.yml
```

## Limitations
 - Fixed Schema: Schema is fixed and cannot be changed in the current implementation
 - Querying interface: Querying is through http POST request, no cli or web-ui
 - Slow build time: Rust release build takes around 5 minutes
 - No Log Pruning: architecture prioritises storing logs and processing them later hence logs maybe stored in the disk, currently no implementations for cleaning, can be implemented in future versions
 - SQL Injection: No input sanitizations are being done, so SQL injection is possible


## Possible bugs
 - seems like the number of files being open does not go down fast enough even though `drop` is being called on every file open, so i've increased the ulimit to the maximum for the process
