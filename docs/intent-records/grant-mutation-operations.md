$id-3269280658856461
title: Independent object mutation operations
date: 2026/11/10
source: @ottojung
kind: requirement

The grant operations `create`, `modify`, and `delete` are independent. `create` permits POST to a previously absent key with initial bytes; `modify` permits PUT to an existing key; `delete` permits DELETE of an existing key. No mutation permission implies another. Grants may delegate any subset of their operations. Other operations remain `list`, `delegate`, and `subscribe`. No generic `write` operation exists.
