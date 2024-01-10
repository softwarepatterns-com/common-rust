# Bus

An event bus within a Rust application. Low-cost and low-memory.

Subscriptions create a directed-graph. Published data follows the directed graph to another function.

```
var bus = Bus::new();

bus.on('metrics.#', function (msg) {
  console.log(msg);
});

channel.emit('metrics.page.loaded', 'hello world');
```

## Subscribing

`*` -- matches exactly one word.
`#` -- matches zero or more words.
