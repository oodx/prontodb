# ProntoDB Admin CLI Documentation

```
    ____                  __       ____  ____
   / __ \________  ____  / /_____  / __ \/ __ )
  / /_/ / ___/ _ \/ __ \/ __/ __ \/ / / / __  |
 / ____/ /  /  __/ / / / /_/ /_/ / /_/ / /_/ /
/_/   /_/   \___/_/ /_/\__/\____/_____/_____/


```

actually we need to build some core modules first, we have objects that need crud management with a clear public api
   before we build any integration patterns. BUT FIRST FIRST some fundamentals. ive updated the structure a bit. we
  are going to start with both rsb and hub. hub is our shared repo to have a single store of dependencies. hub will
  host any libs that have more than 3 shared deps, others have to stay in their own repo you can have china try to run
   `xrepos usage` and see if any of the packages used by pronto are within the 3 or more range, to request hub to
  include it. then youll want to wire in hub and use its features in docs/ref/HOWTO_HUB.txt is a guide, have china
  read it and give you a plan for using it. then make sure we are using the GITHUB version of RSB instead of the
  local, since the local one changes a lot.


 ok next lets iterate on our plan to build up the sqlite interface through a collection of CRUD/admin interfaces. ive
   stubbed out my preferred stucture for the project and the main entry points already use some verson of RSB just for
   a note. what we want to do is create a standard crud interface that will be the same for all objects that need
  crud, for example sqlite base,tables,records to start. ive created lib/adpt (adapter) and in that the sqlite
  adapters for now i have the MODULE_SPEC setup with mod utils and base for the CRUD interface for base. what i would
  do is create a CRUD_SPEC to define what the standard interface pattern looks like so everything that needs crud can
  follow it, and create a CRUD trait or whatever in rust to enforce the CRUD pattern that struct/types have to
  implement. make this a general CRUD interface, you can put the generic crud stuff in the core/ module. but first
  plan it. the idea is for low level crud we want to create an admin cli (bin/cli) that will allow us to interface
  with the low level crud commands directly. the cli should use modern RSB globals,cli, and host modules etc. in
  docs/ref i dropped the features folder which shows all the mainline module features available in RSB (latest). have
  china add this CRUD information/requirements to our current roadmap/tasks, then she should create a quick overview
  of the features docs, theres a lot of them, but focus on the ones you need for the admin cli, i think that will be
  GLOBAL, HOST, STRINGS, DEV(if you want pty wrappers), OPTIONS for sure, maybe PARAMs for bash like access to the
  global store, potentially FS or colors. in order to use RSB you'll want a proper sanity test to make sure the
  feautres of RSB you want to use are working correctly. have china add this to your task list as well and any
  documents for this project, like the START or QUICK_REF need to articulate these required base patterns. any usage
  of RSB must have a sanity test for that RSB feature. this is a lot but help china breakdown her own tasks here, and
  where you can help, do your work in parallel or alternatively ask tina to help with RSB concerns since RSB+Testing
  is her forte
