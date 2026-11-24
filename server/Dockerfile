# MyRoots Server in a container (PHP + Apache + SQLite).
#   docker compose up -d      → http://localhost:8080
FROM php:8.3-apache
RUN a2enmod rewrite headers \
 && sed -ri 's/AllowOverride None/AllowOverride All/g' /etc/apache2/apache2.conf
COPY --chown=www-data:www-data . /var/www/html/
RUN rm -rf /var/www/html/data && mkdir -p /var/www/html/data && chown www-data:www-data /var/www/html/data
VOLUME /var/www/html/data
