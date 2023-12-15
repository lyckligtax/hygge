siehe sans io und scoped vs manual context

Um funktionen auf unseren Services aufzurufen müssen zur Zeit immer ein Context mitgegeben werden.
Sollte der Service einmal angepasst werden muss er an allen Stellen korrigiert werden.

Idee:
Anstatt den Requesthandlern den Service rauszureichen nehmen wir einen Client welcher für jeden Request neu generiert wird. Dieser kann dann schon den passenden Kontext intern verwenden und wir müssen uns keinerlei Gedanken mehr darum machen.

Das lässt sich dann auch sehr gut wegmocken

Wenn der Service auf einmal ausgelagert und als micro service verwenet wird muss sich die Verwendung des ganze gar nicht anpassen sondern es reicht an einer Stelle den Client zu ändern.

Sollte ein Service keinen lokalen State vorhalten so kann er natürlich sein eigener Client sein.

Frage: wie groß ist der Perfomeanceverlust immer einen neuen Service zu generieren? Vernachlässigbar wenn er eh nur den eh verwendeten context speichert? wenige Byte mehr vielleicht. Falls es nicht sogar eh wegkompiliert wird.

Macht es das problem des herumreichens von context besser? oder verlagert es das probelm nur?
interface kommt dann zumindest ohne ctx aus
wie kann ein service dann auf einen anderen zugreifen? darf er dass dann?

kann ich eine sqlx transaction in mehrere clients geben ohne dass etwas kaputt geht?