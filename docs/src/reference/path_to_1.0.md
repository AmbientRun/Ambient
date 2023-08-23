# The path to 1.0

The development of Ambient can be split up into two rough periods; pre-1.0 and post-1.0.

Pre-1.0, which we're currently in, involves heavy iterations on the API and services. This
is a period where we experiment with things, where we take big and wide decisions, and where
we're not afraid to throw things out when we find better ways of doing things. During this
period you can expect fairly frequent breaking changes to the API and services. Very little
effort will be put into maintaining backwards compability.

Post-1.0 this will reverse. After 1.0 we will strive to maintain maximum backwards compability,
and very rarely introduce breaking changes. In the post 1.0 era you should expect much greater
long-term stability, and you should be able to trust that what you build will work for as long
as you're using Ambient as a service.

So when will we reach 1.0? These are the rough things that we think need to be completed before
we can do that:

- [ ] Guest side programmable graphics (user shaders)
- [ ] Performance roundup; we should take one big last look at how api's are structured to figure
  out if there are any broad sweeping changes we need to make to maximize performance
- [ ] One at least medium sized game needs to have been built with the API, to work out the worst kinks

Once those things are in place, we'll set a date for the 1.0.
