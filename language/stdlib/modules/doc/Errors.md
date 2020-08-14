
<a name="0x1_Errors"></a>

# Module `0x1::Errors`

### Table of Contents

-  [Function `make`](#0x1_Errors_make)
-  [Function `invalid_state`](#0x1_Errors_invalid_state)
-  [Function `requires_address`](#0x1_Errors_requires_address)
-  [Function `requires_role`](#0x1_Errors_requires_role)
-  [Function `requires_privilege`](#0x1_Errors_requires_privilege)
-  [Function `not_published`](#0x1_Errors_not_published)
-  [Function `already_published`](#0x1_Errors_already_published)
-  [Function `invalid_argument`](#0x1_Errors_invalid_argument)
-  [Function `limit_exceeded`](#0x1_Errors_limit_exceeded)
-  [Function `internal`](#0x1_Errors_internal)
-  [Function `custom`](#0x1_Errors_custom)

Module defining error codes used in Move aborts throughout the framework.

A
<code>u64</code> error code is constructed from two values:

1. The *error category* which is encoded in the lower 8 bits of the code. Error categories are
declared in this module and are globally unique across the Libra framework. There is a limited
fixed set of predefined categories, and the framework is guaranteed to use those consistently.

2. The *error reason* which is encoded in the remaining 54 bits of the code. The reason is a unique
number relative to the module which raised the error and can be used to obtain more information about
the error at hand. It is mostly used for diagnosis purposes. Error reasons may change over time as the
framework evolves. TODO(wrwg): determine what kind of stability guarantees we give about reasons/
associated module.


<a name="0x1_Errors_make"></a>

## Function `make`

A function to create an error from from a category and a reason.


<pre><code><b>fun</b> <a href="#0x1_Errors_make">make</a>(category: u8, reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Errors_make">make</a>(category: u8, reason: u64): u64 {
    (category <b>as</b> u64) + (reason &lt;&lt; 8)
}
</code></pre>



</details>

<a name="0x1_Errors_invalid_state"></a>

## Function `invalid_state`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_invalid_state">invalid_state</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_invalid_state">invalid_state</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(INVALID_STATE, reason) }
</code></pre>



</details>

<a name="0x1_Errors_requires_address"></a>

## Function `requires_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_requires_address">requires_address</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_requires_address">requires_address</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(REQUIRES_ADDRESS, reason) }
</code></pre>



</details>

<a name="0x1_Errors_requires_role"></a>

## Function `requires_role`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_requires_role">requires_role</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_requires_role">requires_role</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(REQUIRES_ROLE, reason) }
</code></pre>



</details>

<a name="0x1_Errors_requires_privilege"></a>

## Function `requires_privilege`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_requires_privilege">requires_privilege</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_requires_privilege">requires_privilege</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(REQUIRES_PRIVILEGE, reason) }
</code></pre>



</details>

<a name="0x1_Errors_not_published"></a>

## Function `not_published`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_not_published">not_published</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_not_published">not_published</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(NOT_PUBLISHED, reason) }
</code></pre>



</details>

<a name="0x1_Errors_already_published"></a>

## Function `already_published`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_already_published">already_published</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_already_published">already_published</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(ALREADY_PUBLISHED, reason) }
</code></pre>



</details>

<a name="0x1_Errors_invalid_argument"></a>

## Function `invalid_argument`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_invalid_argument">invalid_argument</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_invalid_argument">invalid_argument</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(INVALID_ARGUMENT, reason) }
</code></pre>



</details>

<a name="0x1_Errors_limit_exceeded"></a>

## Function `limit_exceeded`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_limit_exceeded">limit_exceeded</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_limit_exceeded">limit_exceeded</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(LIMIT_EXCEEDED, reason) }
</code></pre>



</details>

<a name="0x1_Errors_internal"></a>

## Function `internal`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_internal">internal</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_internal">internal</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(INTERNAL, reason) }
</code></pre>



</details>

<a name="0x1_Errors_custom"></a>

## Function `custom`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_custom">custom</a>(reason: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Errors_custom">custom</a>(reason: u64): u64 { <a href="#0x1_Errors_make">make</a>(CUSTOM, reason) }
</code></pre>



</details>
