![cdlogo](https://carefuldata.com/images/cdlogo.png)

# serotinous-cone

<b>serotiny</b> - (botany term) <i>following</i> or <i>later</i>

The nodes "follow" each other in a granular fashion as per our orchestration. 

This repository contains IaC and templating for building K3S single node clusters on Alpine Linux.
Additional nodes can be added, but the intent of this design is to keep each node separate and self contained.
Nodes can be standalone, or have a detached replica in another geographic region with DNS/GSLB failover between them,
or of course larger multi-node clusters can be formed. But the point of this design pattern is not to need larger
clusters, to reduce network latency, compute costs, and complexity, simplifying security and performance.

Features of the serotinous-cone nodes:

- hardened Traefik
- Ansible installation and configuration
- microservice templating
- certbot light-weight and scriptable PKI
- manifest templates

### UFW (near Alpine default)

The simple firewall rules are possible because each node is self contained, only SSH and HTTPS need to be exposed. It could be as restricted as only port 443 TCP if no further administration action is needed, but typically we'll want 22 TCP for SSH and then allow all ports (or at least 1443, the kubernetes API) can be used for administrative functions. 

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
The Traefik Ingress example will redirect HTTP (80) to HTTPS (443).

### Traefik, modern TLS modes

The default Traefik exposes a self signed certificate and weaker ciphers. There are three sections at the top of the `morph_manifest.yml__template` that harden Traefik up.
Tune and refine as needed. Note that the settings here are based on high standards, not backwards compatibility with legacy systems.

The example provides "TEMPLATE.com" as an example, update each occurrence of TEMPLATE to the name being utilized. Note that virtually any number of Pods can use 80/tcp,
and any number of FQDNs can be used on the same node without conflict. This is one of the huge strengths of kubernetes Ingress.

There are some security headers added, feel free to adjust and expand from there.

### Just flannel, because size

This design focus on compact and light-weight kubernetes. This is for security, costs, reliability, and ease. When working with clusters this granular and small, Flannel actually shines.
I would normally advocate for Calico or Cilium, but for this these little nodes, Flannel is perfect. The template leverages Ingress and keeps all the microservices within host,
so no need to deal with inter-node optimization or mesh security. 

If you want to switch out flannel for a more fully featured CNI plugin, change the k3s install flags to include `--flannel-backend=none --disable-network-policy` and then afterwards install the 
appropriate CNI plugin. Most of my other templates for K3s do this, installing Calico for the CNI plugin. See more examples of K3S + Calico here: https://github.com/jpegleg/k3s-dragon-eggs/

### HostPath and shell script, oh my

HostPath Volume mounts are often a bad thing, an anti-pattern in cloud native design. But when working at super small scale, it is great for productivity, speed, security, and reliability: just push out files over SSH, easy as that.
The certbot and scripts PKI is lighter weight than running a full fledged cloud native solution, and interestingly is not only more reliable but scales surprisingly well. 

If HTTP ACME challenges are used and there are multiple DNS A records going to multiple cones, then the web data (/srv/persist) will need to be syncronized between nodes in near real time in order to complete the ACME challenge.
There are alternate challenge types based on DNS records that can be used, otherwise link the storage.

#### But we don't <i>have</i> to have HostPath here! We can use other storage mechanisms instead, no problem.

Just because this template defaults to using a HostPath setup on `/srv/persist`, doesn't mean everything is stuck that way. Switching out to alternative storage mechanisms works fine! 

If we do want to keep the HostPath, but want to sync the data to a few other cones in a simple and secure way, one option is sshfs. This works fine for two nodes that need to sync up on web material and PKI files.

### Certbot and scripted ACME

In this design, regardless of whether we use Traefik's ACME functionality, we also include certbot so that TLS certificates can be issued and renewed in a more flexiable and reliable way.
While in-cluster ACME has plenty of advantages and on paper is better, in practice we might desire a mechanism to handle renewals outside of the cluster. This can be for orchestration reasons, such
as having multiple clusters in roundrobin DNS (HTTP challenges would fail), and the fact that DNS challenges in Traefik are prone to issues when more than one certificate is involved.

The acme_wrapper is a script to take the output of certbot, clean it up (remove root from chain pem so the file used is leaf + intermediate), and place them in the loading zone directories. The acme_wrapper then calls the loader.sh,
that deletes the kubernetes TLS secret and replaces it with the data from the loading zone directories.

The certbot renewal itself is done prior to the acme_wrapper execution, whether that is from crontab, run manually, or orchestrated Ansible, etc etc. I'll likely include some examples of this certbot execution part eventually.

### SDLC glory

Patching can be full of surprises, especially for Kubernetes and Alpine. Rather than patching or changing the node or cluster after it is in use, in this design pattern we just keep buliding new ones. Create new servers (such as with OpenTofu/Terraform and Packer), and refine them, deploy the latest code, check everything out, then point traffic over via DNS/GSLB when it is ready. When everything is well validated, then the old node/s can be removed from DNS/GSLB and then deleted. This keeps upgrades and patching flowing smoothly and without surprises.

Another great aspect of using K3S is that it works on other linux distros, so much of the configuration is portable if we want to either not use Alpine, or use something in addition to Alpine. Developers can run replicas of most of the functionality locally (minus the PKI, using self signed certs instead for dev).


### Microservice templates, tiny rust apps that lean on Traefik Ingress

In this design pattern, we can make small microservices that don't need to hold water on their own against the internet in terms of TLS. They sit behind Traefik, leveraging Traefik for TLS.
Because this design is single node, service-to-service traffic within the cluster scope is entirely within the same kernel. This enables us to shed some of the complexity
of TLS management, as that is handled granularly at the platform (k3s Traefik) level. While this technically weakens TLS to terminate at Traefik, it reduces costs, simplifies operations, and is compute/cost effective.

There is a template for a simple web server within the `morph_micro_template` directory. The "morph micro" is an Actix web server that serves static web files, and includes support to complete ACME HTTP challenges.
The micro morph acts in place of a web server, serving up whatever web code is desired. The web code in the template is mounted to the node so that website changes can happen by deploying files to the node, or desired storage system/s. Having 
the web material separate means that the morph micro service rarely needs to change, if ever. One reason to customize the morph micro is to add additional controls and routes to specific files and paths. To do this,
copy the index funciton, give it a new name and path, customize the new function as needed, and then add in a new .service(YOURTHING) with YOURTHING being the new function name. Then use (cross) to compile with musl libc, then docker/podman build a new OCI image from `scratch` to run the new statically linked binary. Then save the container to a tar file and insert the tar file via k3s ctr image import, or otherwise deploy to the registry it can be pulled from.

At 12MB total for the morph micro, that single container image can be utilized to serve up many websites and front-ends. This enables web code to avoid needing to re-invent and build new container images, instead the micro morph can handle all of them, with incredible performance anad reliability as well as security, and the web code just needs to be synced to the (storage) mount location for that website. The kubernetes manifests segment each website or scope, enabling completely granular yet centralized management.

Alternatively to adding (URI context) routes with micro morph rust changes, Traefik can also be used to add routes. Traefik can act as a service gateway beyond SNI matching, and also do URI context based routing, and other types of gateway functionality. There is no need to writing a new gateway microservice here, Traefik can handle that. Expand the manifest to include any additional Traefik configurations needed for that cutomization.https://github.com/jpegleg/k3s-dragon-eggs/tree/main

The micro morph is a good example for the serotinous cone design pattern, but any container could be used here.

### Registry vs "air-gapped" tarballs

The template has tarballs made from containers used to import directly. This is a common pattern for air-gapped installations, and can be useful if the container image isn't supposed to change once the cluster is running. In scenarios where the container image needs to change frequently during the life of the cluster, it is possibly better to follow the standard designs of container registries. Adding a registry can be a significant attack surface and increase costs, but empowers developers to make changes to container images that can be configured to be pulled automatically. The serotinous-cone doesn't <i>need</i> a private registry and can be built simply, but <i>can</i> be configured to use a registry instead or in addition to the tarball import. Even if the containers need to change frequently, there may be cases where the tarball approach is still more secure and/or more cost effective.



## More coming to the README soon!
