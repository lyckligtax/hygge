currently we manually create a sort of context that we giive out to different services
these services might give the ctx further down

we do this because we want to control the way IO is handled
this is sort of dependency injection and allows us to better test

currently this is done by defining kind of generic params so there is no overhead when compiling this down

this is more explicit than scoped context

---
scoped context would make interfacing easier
function would not need to take an extra param just to pass it down

question: how bad is it actially? does is really matter much?

---
i guess it does not , define why we do not want this in full

MANUAL context is easier to reason about