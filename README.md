![cdlogo](https://carefuldata.com/images/cdlogo.png)

# serotinous-cone

<b>serotiny</b> -<i> to follow</i>

This repository contains IaC and templating for building K3S single node clusters on Alpine Linux.
Additional nodes can be added, but the intent of this design is to keep each node separate, yet
to have more than one in different geographic regions with DNS/GSLB failover between them. The
nodes "follow" each other in a granular fashion as per our orchestration.

Features of the serotinous-cone nodes:

- hardened Traefik
- Ansible installation and configuration
- microservice templating
- acme.sh light-weight and scriptable PKI
- manifest templates

### UFW (near Alpine default)

The simple firewall rules are possible because each node is self contained, only SSH and HTTPS need to be exposed.

```
# 22 is already open by default in Alpine: ufw allow in 22/tcp
ufw allow in 443/tcp
ufw allow from $ADMINHOST to any
ufw reload
```

With those three steps in place, the firewall should look like this:

```
$ ufw status
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere                  
Anywhere                   ALLOW       $YOURADMINHOST             
443/tcp                    ALLOW       Anywhere                  
22/tcp (v6)                ALLOW       Anywhere (v6)             
443/tcp (v6)               ALLOW       Anywhere (v6)          
```

### Traefik, modern TLS modes

The default Traefik exposes a self signed certificate and weaker ciphers. There are two blocks at the top of the `morph_manifest.yml__template` that harden Traefik up.
Tune and refine as needed. Note that the settings here are based on high standards, not backwards compatibility with legacy systems.

There are some security headers added, feel free to adjust and expand from there.


### Just flannel, because size

This design focus on compact and light-weight kubernetes. This is for security, costs, reliability, and ease. When working with clusters this granular and small, Flannel actually shines.
I would normally advocate for Calico or Cilium, but for this these little nodes, Flannel is perfect. The template leverages Ingress and keeps all the microservices within host,
so no need to deal with inter-node optimization or mesh security. 

### HostPath and shell script, oh my

HostPath Volume mounts are often a bad thing, an anti-pattern in cloud native design. But when working at super small scale, it is great for productivity, speed, security, and reliability: just push out files over SSH, easy as that.
The shell script PKI is lighter weight than running a full fledged cloud native solution, and interestingly is not only more reliable but scales surprisingly well. 

## More coming to the README soon!
