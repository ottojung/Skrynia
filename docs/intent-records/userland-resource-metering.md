$id-1891053258346578
title: Resource metering through custom userland virtual machines
date: 2026/11/26
source: @ottojung
kind: requirement

Implement resource measurement, metering, execution budgets and quota
policies in custom userland VMs running on Skrynia, not in the native
kernel or through Wasmtime's resource-limiting facilities.

Do not design the new kernel ABI around Wasmtime fuel, instruction
budgets, epoch-based deadlines, store resource limits or aggregate
fuel transfer between nested evaluations. Remove existing
Wasmtime-specific metering and execution-limit requirements from the
target design. Native eval executes trusted or explicitly authorized
WASM; software requiring measured or bounded execution uses a suitable
userland VM that controls its own instruction execution and mediated
resource operations.

The choice of userland VM, cost model, measurement units, enforcement
and accounting of foreign calls is outside the kernel contract.
Kernel memory safety, bounds checking, correct error handling, and
ordinary host resource/allocation failures remain distinct from
application resource metering. No guarantee of termination or bounded
resource consumption is implied for unrestricted native eval.

This supersedes previous proposals for kernel-level fuel accounting,
host-work budgets and Wasmtime execution limits. It describes the
desired design, not the current implementation.
