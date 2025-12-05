# Transaction Engine

My naive implementation of a CLI transaction engine that consumes data from a .csv file and produces output to the default console output.

## Usage

The only parameter it accepts is a csv filename.

## Limitation(s) / Note(s)

- Only deposits can be disputed. Deposits basically act as money transfered from another account to the one indicated in the transaction, so when interpreting that as a payment processor, it makes sense only they can be disputed. This behaviour can be changed easily by branching in the implementation of the dispute method of the Account implementation.

- The current engine implementation can be easily extended to consume input and produce output somewhere else. Runner could be transformed to a trait and the different runner implementations could consume data from other sources. (Note in case of a multi-threaded env the accesses to the engine needs to be wrapped in guards, or the data held in the engine needs to be guarded and tagged as send and sync)
