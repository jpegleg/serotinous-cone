![cdlogo](https://carefuldata.com/images/cdlogo.png)

# serotinous-cone

<b>serotiny</b> -<i> to follow</i>

This repository contains IaC and templating for building K3S single node clusters on Alpine Linux.
Additional nodes can be added, but the intent of this design is to keep each node separate, yet
to have more than one in different geographic regions with DNS/GSLB failover between them. The
nodes "follow" each other in a granular fashion as per our orchestration.

Features of the serotinous-cone nodes:

- hardened Traefik
- complete IaC
- microservice templating
- acme.sh light-weight and scriptable PKI
- manifest templates


## More coming to the README soon!
