![cdlogo](https://carefuldata.com/images/cdlogo.png)

# serotinous-cone

<b>serotiny</b> - (botany term) <i>following</i> or <i>later</i>

This repository contains IaC and templating for building K3S single node clusters on Alpine Linux.
Additional nodes can be added, but the intent of this design is to keep each node separate, yet
to have more than one in different geographic regions with DNS/GSLB failover between them. The
nodes "follow" each other in a granular fashion as per our orchestration.

Features of the serotinous-cone nodes:

- hardened Traefik
- Ansible installation and configuration
- microservice templating
- certbot light-weight and scriptable PKI
- manifest templates

### UFW (near Alpine default)

The simple firewall rules are possible because each node is self contained, only SSH and HTTPS need to be exposed.

```
# 22 is already open by default in Alpine: ufw allow in 22/tcp
ufw allow in 443/tcp
ufw allow from $ADMINHOST to any
ufw reload
```

With those three steps in place, the firewall should look like this (with the placeholder variable being the actual IP of the admin/bastion machine):

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

To enable ACME cert renewals with HTTP challenges, and unencrypted ingress in general, additionally add port 80: `ufw allow 80/tcp`.

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
The certbot and scripts PKI is lighter weight than running a full fledged cloud native solution, and interestingly is not only more reliable but scales surprisingly well. 

If HTTP ACME challenges are used and there are multiple DNS A records going to multiple cones, then the web data (/srv/persist) will need to be syncronized between nodes in near real time in order to complete the ACME challenge.
There are alternate challenge types based on DNS records that can be used, otherwise link the storage. I'll likely add some storage configurations to this repo eventually.

### Certbot and scripted ACME

In this design, regardless of whether we use Traefik's ACME functionality, we also include certbot so that TLS certificates can be issued and renewed in a more flexiable and reliable way.
While in-cluster ACME has plenty of advantages and on paper is better, in practice we might desire a mechanism to handle renewals outside of the cluster. This can be for orchestration reasons, such
as having multiple clusters in roundrobin DNS (HTTP challenges would fail), and the fact that DNS challenges in Traefik are prone to issues when more than one certificate is involved.

The acme_wrapper is a script to take the output of certbot, clean it up (remove root from chain pem so the file used is leaf + intermediate), and place them in the loading zone directories. The acme_wrapper then calls the loader.sh,
that deletes the kubernetes TLS secret and replaces it with the data from the loading zone directories.

The certbot renewal itself is done prior to the acme_wrapper execution, whether that is from crontab, run manually, or orchestrated Ansible, etc etc. I'll likely include some examples of this certbot execution part eventually.


## More coming to the README soon!
