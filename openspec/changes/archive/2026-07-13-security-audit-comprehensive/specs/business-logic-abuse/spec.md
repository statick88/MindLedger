# Business Logic Abuse Specification

## Purpose

Verify that accounting invariants, transaction validation, and appointment state machine resist abuse and manipulation. Covers OWASP A08:2021.

## Requirements

### Requirement: Transaction Amount Validation

The system MUST reject negative, zero, and overflow transaction amounts atomically — no partial database writes.

#### Scenario: Negative transaction amount

- GIVEN: TypeScript DTO with negative amount in `lineas[]`
- WHEN: Sent to accounting IPC command
- THEN: Rust backend rejects atomically
- AND: Error returned to frontend
- AND: No database write occurs

#### Scenario: Overflow amount

- GIVEN: Amount exceeding i64::MAX
- WHEN: Sent to accounting IPC command
- THEN: Validation error returned
- AND: Database state unchanged

### Requirement: Accounting Equation Invariant

The accounting equation (sum(debito) = sum(credito)) MUST be enforced at creation time. Imbalanced asientos MUST be rejected.

#### Scenario: Debit-credit imbalance

- GIVEN: Asiento where sum(debito) != sum(credito)
- WHEN: Sent to `create_asiento` command
- THEN: Rejected with validation error
- AND: Accounting equation is preserved
- AND: No partial writes occur

### Requirement: Appointment State Machine Integrity

The appointment state machine MUST reject invalid transitions. Terminal states MUST NOT allow re-entry.

#### Scenario: Invalid state transition

- GIVEN: Appointment in "Cancelada" state
- WHEN: Transition to "Realizada" attempted
- THEN: State machine rejects invalid transition
- AND: Error returned, state unchanged

#### Scenario: Terminal state re-entry

- GIVEN: Appointment in "Realizada" (terminal) state
- WHEN: Transition to any state attempted
- THEN: Transition is rejected
- AND: Appointment remains in terminal state
