# textabus

This is a Rust/axum interface to Twilio for obtaining bus information from Winnipeg Transit’s [Open Data Web Service](https://api.winnipegtransit.com/), it’s meant to fill the gap since Winnipeg city council axed [BUStxt](https://web.archive.org/web/20190630175528/https://winnipegtransit.com/en/schedules-maps-tools/transittools/bustxt-user-guide/) in 2020 to save $45k/yr while continuing hand the [murderous](https://www.cbc.ca/news/canada/manitoba/officer-involved-shooting-winnipeg-1.7072134) Winnipeg Police Service over $300mil/yr.

It’s nascent but can hopefully eventually replicate most of BUStxt, although it appears the beloved feature of being able to txt a bus number to see its scheduled arrivals is not supported by the API.
