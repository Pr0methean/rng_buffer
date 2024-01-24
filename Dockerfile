FROM intel/oneapi-basekit:devel-ubuntu22.04
COPY ./vtune_all.sh /
COPY ./os /
COPY ./buffer_size_* /

USER root

RUN cd /opt/intel/oneapi/vtune/2024.0/sepdk/src
RUN ./insmod-sep -g vtune -pu
RUN ./boot-script --install
RUN cd /

RUN apt-get install -y --no-install-recommends python build-essential curl unzip

RUN curl --insecure "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
RUN unzip awscliv2.zip
RUN ./aws/install

LABEL authors="hennickc"

ENTRYPOINT ["/vtune_all.sh"]