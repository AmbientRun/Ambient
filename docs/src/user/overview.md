# Overview of Ambient

Let's start with a rough overview of Ambient to give you an idea of how it works.

## The database (ECS)

The most central thing in Ambient is the [ECS](../reference/ecs.md) "World". You can think of it
as a database which stores everything in your application.

The World is a collection of entities, an entity is a collection of components and a component is a
(name, value) pair. For example, you could have an entity with two components;

```yml
entity 1932:
    - translation: (5, 2, 0)
    - color: (1, 0, 0, 1)
```

If you compare this to a traditional database, you can think of entities as rows and components as columns,
except that there are no tables; you can just attach any component to any entity you'd like.

## Client/server

The next thing to know is that Ambient is built around a client/server architecture. Both
the server and each client have their own ECS World instance.

The server ECS World is then automatically replicated to all clients' Worlds, so that they
contain perfect copies of the servers World. Any change made to the ECS World on the server side
will also automatically be done on the clients World's.

The replication is one-way though, so making a change on the client side to your World will
_not_ be replicated to the server. To make changes from a client, you have to send a
[message](../reference/networking.md#messaging) from the client to the server, and the apply
any changes you want to the server World there.
