# ambient_friendly_id

This code was adapted from `friendly_id` (MIT/Apache2):

<https://github.com/mariuszs/friendly_id/blob/d691da682027b84eddd771d7adac0c0dc2563a35/src/base62.rs>

The crate depends on `failure`, which is unmaintained. As we only need to generate some random IDs,
we can make do by reimplementing it.
