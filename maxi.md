
# Vorgeschlagene Herangehensweise

## Vorbereitung
-   Basierend auf Hardwarekosten beste EC2 Instanz heraussuchen
    -   Storage pro Dollar ist die wichtigste Metrik
    -   Siehe <http://www.vldb.org/pvldb/vol14/p1606-leis.pdf>, Table 1
    -   Daten basierend auf <https://instances.vantage.sh>
-   Related Work, Blog Posts etc. bzgl. Blob Storage Implementierung suchen
-   S3 service zusammenfassen (wie viele 9&rsquo;s availability, &#x2026;)

## Iteration 1
-   Single-Node anfangen: RAID 5/6 auf HDDs erstellen
-   In-Memory Metadatenservice (Key->Value store)
-   Evtl. bereits testweise &ldquo;near-storage computation&rdquo; implementieren
-   Benchmarken: Wo ist der Bottleneck? Storage/Network? Verbesserungspotential?

## Iteration 2
-   Metadatenservice auf extra (low pro-RAM cost, siehe Vorbereitung) Instanz
-   Aber Objekte liegen immer noch auf einer dedizierten Node
    -   HDD Nodes haben jetzt vermutlich eher eine low-level API, also `GET objecthash?bytes=100-1500`
    -   Überlegen, ob/wie man near-storage computation noch auf nodes verteilen kann
-   Benchmarken: Wo ist der Bottleneck? Networking? Storage Nodes? Metadata Node?

## Iteration 3
-   Objekt wird beim Schreiben auf mehrere Nodes aufgeteilt
-   Beim Lesen müssen Objekte wieder zusammengebaut werden
-   Überlegen, ob man eine &ldquo;direct streaming&rdquo; API anbieten will, bei der die Metadaten-Node entlastet wird, indem sie nur noch Direktverbindungen zw. Client und Storage Nodes vermittelt
-   Benchmarken: Wo ist der Bottleneck? Storage/Network/Metadata?

## Iteration 4
-   Implementieren von weiteren &ldquo;near-storage computation&rdquo; Funktionen
-   Vorteil von storage-naher Computation benchmarken

## Orga
-   Anfang Mitte Oktober
-   Praesentation am Ende
-   AWS
