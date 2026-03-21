## Attribut
### CELL
#### VISITED
- `unknown` --> Zelle wurde bisher noch nicht gesehen von Micromouse
- `discovered` --> Sensor hat in die Zelle reingesehen, wurde von der Micromouse aber noch nicht besucht
- `visited` --> Micromouse war physisch innerhalb des Feldes
#### COST
- `NaN` --> Zell-kosten wurden vom Algorithmus für diese Zelle noch nicht bestimmt
- x (Wert im Bereich von 0-100)

#### IS_INTERSECTION
- `true` --> Die Zelle ist eine Kreuzung
- `false` --> Die Zelle ist keine Kreuzung (oder wurde noch nicht als eine solche identifiziert) 

### WALL
#### EXISTS
- `true` --> Wand wurde durch Kollision mit einer Messung erkannt
- `false` --> Messung hat Ort der potentiellen Wand durchquert, keine Wand vorhanden
- `unknown` --> Bisher noch nicht mit potentieller Wand interagiert --> Könnte existieren

### PATH
#### COMPLETION
- `future` --> Pfad wurde bisher noch nicht angefangen
- `in_process(x,y)` --> Wobei`(x,y)` die Koordinaten darstellen, bis zu denen der Pfad abgehandelt wurde
- `completed` --> Pfad wurde schon vollständig abgearbeitet

#### CERTAINTY
- `required` --> Der Algorithmus ist sicher, dass dieser Pfad genommen werden muss
- `optional` --> Der Pfad ist nicht unbedingt notwendig, sondern nur eine der Alternativen, die berücksichtigt wird
- `rejected` --> Der Pfad war mal optional, ist jetzt aber nicht mehr aktuell



