

remove any custom error implementation use standard-error crate... make sure to have meaningful err messages..

update extenstion UI panel (devtools) to show list of errors (distinct by codes)

heres the thing, capture rs has good model for consolerecord and harrecord

just have a export trait in exporter module under internal thats take self and retirms Resilt<> from prelude

implement this for consolerecord

 such that it emits otel logs to &settings.otel_collector_endpoint

implement this for harrecord 

 it emits traces based on timing information.. e.g. for a post request i want spans for ssl, request sent, response, headers, complete.. do not assume json in body.. send bytes as is.. 

 remove any reinventions be it otel attributes or unnecessary enums, have straightforwardx.


dont stop until it compiles... 

add tests with a fixture json sample for har and console exports

yeah that takes care of traces and logs being sent over in exporter.. pub use whjatss ned and import in hjandler


NO STUPID UNCLE BOB CLEAN CODE... i dont want unnecessary function splits and types... stay procedural where possile.. use functional chains only if it makes sense.. use interfacts like the one i mentioned for export.. I DONT WANT TO SEE ANYTHIGN ERXPLICITLY CALLED Strategy .e.g. ... design patterns is not the goal, just the means
