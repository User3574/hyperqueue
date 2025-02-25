## Resource management

Resource management serves for defining arbitrary resources provided by workers and
also corresponding **resource requests** required by tasks. HyperQueue will take care of matching
task resource requests so that only workers that can fulfill them will be able to execute such tasks.

Some generic resources are [automatically detected](#automatically-detected-resources); however,
users may also define their own resources.

From version 0.13.0, CPUs are also managed as other resources, but they have still some extra functionality;
therefore, there is special chapter about [CPU resources](cresources.md). 

!!! important

    Resources in HyperQueue exist on a purely logical level. They can correspond to physical
    things (like GPUs), but it is the responsibility of the user to make sure that this correspondence
    makes sense. With exception of CPUs, HyperQueue by itself does not attach any semantics to resources,
    they are just numbers used for scheduling.


## Worker resources

Each worker have one or mores resources attached. Each resource is a **resource pool**
identified by a name. A resource pool represents some resources provided by a worker; each task can
then ask for a part of resources contained in that pool.

There are two kind of resource pools:

* **Indexed pool**: This pool represents an enumerated set of resources represented by integers.
Each resource has its own identity. Tasks do not ask for specific values from the set, they just specify
how many resources do they require and HyperQueue will allocate the specified amount of resources
from the pool for each task.

    This pool is useful for resources that have their own identity, for example individual GPU or
    FPGA accelerators.

    HyperQueue guarantees that no individual resource from the indexed pool is allocated to more than
    a single task at any given time and that a task will not be executed on a worker if it does not
    currently have enough individual resources to fulfill the [resource request](#resource-request)
    of the task.

    Indexed pool can ale defined with **groups** where indices live in separated groups. Task may
    then ask for different allocation policies (e.g. use resources from the same or different groups).
    The main purpose of this is to capture NUMA architectures, each group then represents a socket with cores.

* **Sum pool**: This pool represents a resource that has a certain size which be split into individual
    tasks. A typical example is memory; if a worker has `2000` bytes of memory, it can serve e.g. four
    tasks, if each task asks for `500` bytes of memory.

    HyperQueue guarantees that the sum of resource request sizes of *running* tasks on a worker does
    not exceed the total size of the sum pool.

### Specifying worker resources

You can specify the resource pools of a worker when you start it:

```bash
$ hq worker start --resource "<NAME1>=<DEF1>" --resource "<NAME2>=<DEF2>" ...
```

where `NAMEi` is a name (string ) of the `i`-th resource pool and `DEFi` is a definition of the
`i-th` resource pool. You can define resource pools using one of the following formats:

* `[<VALUE>, <VALUE>, ..., <VALUE>]` where `VALUE` is a non-negative integer. This defines a
   an indexed pool with given values.
* `range(<START>-<END>)` where `START` and `END` are non-negative integers. This defines
   an indexed pool with numbers in the inclusive range `[START, END]`.
* `[[<VALUE>, ..., <VALUE>], [<VALUE>, ..., <VALUE>], ...]` where `VALUE` is a non-negative integer. This
   defines an indexed pool where indices are grouped.
* `<N>x<M>` Creates indexed pool with N groups of size M, indices are indexed from 0,
  (e.g. "2x3" is equivalent to `[[0, 1, 2], [3, 4, 5]`)
* `sum(<SIZE>)` where `SIZE` is a positive integer. This defines a sum pool with the given
  size.

!!! tip

    You might encounter a problem in your shell when you try to specify worker resources, because
    the definition contains parentheses (`()`). In that case just wrap the resource definition in
    quotes, like this:

    ```bash
    $ hq worker start --resources "foo=sum(5)"
    ```

### Automatically detected resources

The following resources are detected automatically if a resource of a given name is not explicitly defined.

* CPUs are automatically detected as resource named "cpus" (more in [CPU resources](cresources.md)).

* Nvidia GPUs that are available when a worker is started are automatically detected under the resource
  name `gpus`. You can use the environment variable `CUDA_VISIBLE_DEVICES` when starting a worker to
  override the list of available GPUs:
  
```bash
$ CUDA_VISIBLE_DEVICES=2,3 hq worker start
```

* RAM of the node is detected as resource "mem" in bytes. 

If you want to see how is your system seen by a worker without actually starting it,
you can start: 

```bash
$ hq worker hwdetect
```

The automatic detection of resources can be disabled by argument ``--no-detect-resources`` in ``hq worker start ...``.
It disables detection of resources other than "cpus";
if resource "cpus" are not explicitly defined, it will always be detected.


## Resource request

When you submit a job, you can define a **resource requests** with the `--resource` flag:

```bash
$ hq submit --resource <NAME1>=<AMOUNT1> --resources <NAME2>=<AMOUNT2> ...
```

Where `NAME` is a name of the requested resource and the `AMOUNT` is a positive integer defining the
size of the request.

Tasks with such resource requests will only be executed on workers that fulfill all the specified
task requests.

!!! important

    Notice that task resource requests always ask for an amount of resources required by a task,
    regardless whether that resource corresponds to an indexed or a sum pool on workers.

    For example, let's say that a worker has an indexed pool of GPUs:
    ```bash
    $ hq worker start --resource "gpus=range(1-3)"
    ```
    And we create two jobs, each with a single task. The first job wants 1 GPU, the second one wants
    two GPUs.

    ```bash
    $ hq submit --resource gpus=1 ...
    $ hq submit --resource gpus=2 ...
    ```

    Then the first job can be allocated e.g. the GPU `2` and the second job can be allocated the GPUs
    `1` and `3`. 


## Requesting all resources

A task may ask for all given resources of that type by specifying ``--resource <NAME>=all``.
Such a task will be scheduled only on a worker that has at least ``1`` of such resource and when a task is executed
all resources of that type will be given to this task. 


## Resource request strategies

When resource request is defined, after the amount you can define allocation strategy:
``--resource <NAME>="<AMOUNT> <STRATEGY>"``.

Specifying strategy has effect only if worker provides indexed resource in groups.
If resource is other type, then strategy is ignored.

When strategy is not defined then ``compact`` is used as default.

* Compact (``compact``) - Tries to allocate indices in few groups as possible in the current worker state.

  Example: ``hq submit --resource cpus="8 compact" ...``

* Strict Compact (``compact!``) - Always allocate indices on as few groups as possible for a target node.
  The task is not executed until the requirement could not be fully fulfilled.
  E.g. If a worker has 4 indices per a group and you ask for 4 indices in the strict compact mode,
  it will always be executed with indices from a single group.
  If you ask for 8 cpus in the same way, it will always be executed with indices from two groups.

  Example: ``hq submit --resource cpus="8 compact!" ...``

* Scatter (``scatter``) - Allocate indices across as many groups as possible in the current worker state.
  E.g. Let us assume that a worker has 4 groups with 8 indices per group, and you ask for 8 cpus in the scatter mode.
  If possible in the current situation, HQ tries to run process with 2 cpus on each socket.

  Example: ``hq submit --resource="8 scatter" ...``


### Resource environment variables
When a task that has resource requests is executed, the following variables are passed to it for
each resource request named `<NAME>`:

* `HQ_RESOURCE_REQUEST_<NAME>` contains the amount of requested resources.
* `HQ_RESOURCE_VALUES_<NAME>` contains the specific resource values allocated for the task as a
comma-separated list. This variable is only filled for indexed resource pool.

!!! tip

    HQ has a special case for a resource named `gpus`. For that resource, it will also pass the following
    environment variables to the spawned task:

    * `CUDA_DEVICE_ORDER` set to the value `PCI_BUS_ID`
    * `CUDA_VISIBLE_DEVICES` set to the same value as `HQ_RESOURCE_VALUES_gpus`


## Resource requests and job arrays

Resource requests are applied to each task of job. For example, if you submit the following:

```bash
$ hq submit --cpus=2 --array=1-10
```
