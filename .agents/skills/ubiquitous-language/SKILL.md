---
name: ubiquitous-language
description: Extract DDD ubiquitous language glossary from conversation, flagging ambiguities and saving to UBIQUITOUS_LANGUAGE.md
when_to_use: user wants to define domain terms, build glossary, or mentions "domain model" or "DDD".
metadata:
  category: analysis
---
# Ubiquitous Language

## When to Use

Extract and formalize domain terminology from current conversation into consistent glossary, saved to local file.

## Process

1. **Scan conversation** for domain-relevant nouns, verbs, and concepts
2. **Identify problems**:
   - Same word used for different concepts (ambiguity)
   - Different words used for same concept (synonyms)
   - Vague or overloaded terms
3. **Propose canonical glossary** with opinionated term choices
4. **Write to `.agents/UBIQUITOUS_LANGUAGE.md`** using format below
5. **Output summary** inline in conversation

## Output Format

Write `UBIQUITOUS_LANGUAGE.md` with this structure:

```md
# Ubiquitous Language

## Order lifecycle

| Term        | Definition                                              | Aliases to avoid      |
| ----------- | ------------------------------------------------------- | --------------------- |
| **Order**   | A customer's request to purchase one or more items      | Purchase, transaction |
| **Invoice** | A request for payment sent to a customer after delivery | Bill, payment request |

## People

| Term         | Definition                                  | Aliases to avoid       |
| ------------ | ------------------------------------------- | ---------------------- |
| **Customer** | A person or organization that places orders | Client, buyer, account |
| **User**     | An authentication identity in the system    | Login, account         |

## Relationships

- An **Invoice** belongs to exactly one **Customer**
- An **Order** produces one or more **Invoices**

## Example dialogue

> **Dev:** "When a **Customer** places an **Order**, do we create the **Invoice** immediately?"
> **Domain expert:** "No — an **Invoice** is only generated once a **Fulfillment** is confirmed. A single **Order** can produce multiple **Invoices** if items ship in separate **Shipments**."
> **Dev:** "So if a **Shipment** is cancelled before dispatch, no **Invoice** exists for it?"
> **Domain expert:** "Exactly. The **Invoice** lifecycle is tied to the **Fulfillment**, not the **Order**."

## Flagged ambiguities

- "account" used to mean both **Customer** and **User** — distinct concepts: **Customer** places orders, while **User** is authentication identity that may or may not represent a **Customer**.
```

## Rules

- **Be opinionated.** When multiple words exist for same concept, pick best one and list others as aliases to avoid.
- **Flag conflicts explicitly.** If term used ambiguously in conversation, call it out in "Flagged ambiguities" with clear recommendation.
- **Only include terms relevant for domain experts.** Skip module or class names unless they have domain meaning.
- **Keep definitions tight.** One sentence max. Define what it IS, not what it does.
- **Show relationships.** Use bold term names and express cardinality where obvious.
- **Only include domain terms.** Skip generic programming concepts (array, function, endpoint) unless they have domain-specific meaning.
- **Group into multiple tables** when natural clusters emerge (e.g. by subdomain, lifecycle, or actor). Each group gets own heading and table. If all terms belong to single cohesive domain, one table fine — don't force groupings.
- **Write example dialogue.** Short conversation (3-5 exchanges) between dev and domain expert demonstrating how terms interact naturally. Dialogue should clarify boundaries between related concepts and show terms used precisely.

  *Example:*
  > **Dev:** "How do I test the **sync service** without Docker?"
  >
  > **Domain expert:** "Provide **filesystem layer** instead of **Docker layer**. Implements same **Sandbox service** interface but uses local directory as **sandbox**."
  >
  > **Dev:** "So **sync-in** still creates a **bundle** and unpacks it?"
  >
  > **Domain expert:** "Exactly. **Sync service** doesn't know which layer it's talking to. Calls `exec` and `copyIn` — **filesystem layer** just runs those as local shell commands."

## Re-Running

When invoked again in same conversation:

1. Read existing `UBIQUITOUS_LANGUAGE.md`
2. Incorporate new terms from subsequent discussion
3. Update definitions if understanding evolved
4. Re-flag any new ambiguities
5. Rewrite example dialogue to incorporate new terms
