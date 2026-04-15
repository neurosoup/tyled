<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.12.0" name="HP" tilewidth="1" tileheight="32" tilecount="2" columns="2">
 <image source="hp.png" width="2" height="32"/>
 <tile id="0">
  <properties>
   <property name="HPBar" type="class" propertytype="tyled::components::markers::HPBar"/>
  </properties>
 </tile>
 <tile id="1">
  <properties>
   <property name="HPBar" type="class" propertytype="tyled::components::markers::HPBar">
    <properties>
     <property name="player_id" type="int" value="1"/>
    </properties>
   </property>
  </properties>
 </tile>
</tileset>
