# First, test home title.
GET http://hygge:3000/

HTTP 200
[Asserts]
xpath "normalize-space(//head/title)" == "Hello world!"