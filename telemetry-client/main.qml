pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtGraphs

import "components"

ApplicationWindow {
    id: mainView
    width: 1280
    height: 720

    visible: true

    color: "#171430"

    Rectangle {
        id: menuRoot

        anchors.left: parent.left
        anchors.leftMargin: 20

        anchors.top: parent.top
        anchors.topMargin: 20

        anchors.bottom: parent.bottom
        anchors.bottomMargin: 20

        width: 65
        height: parent.height

        color: "#211d44"
        radius: 10
        
        ColumnLayout {
            anchors.left:  parent.left
            anchors.right: parent.right

            anchors.verticalCenter: parent.verticalCenter
            spacing: 12

            Rectangle {
                height: 50
                width:  50
                radius: 15
                Layout.alignment: Qt.AlignCenter

                color: "#211d44"

                Image {
                    anchors.centerIn: parent

                    source: "qrc:/dashboard.svg"
                    width: 32
                    height: 32
                    fillMode: Image.PreserveAspectFit
                }
            }

            Rectangle {
                height: 50
                width:  50
                radius: 15
                Layout.alignment: Qt.AlignCenter

                color: "#211d44"

                Image {
                    anchors.centerIn: parent

                    source: "qrc:/live.svg"
                    width: 32
                    height: 32
                    fillMode: Image.PreserveAspectFit
                }
            }

            Rectangle {
                height: 50
                radius: 15
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignCenter

                color: "#211d44"

                Image {
                    anchors.centerIn: parent

                    source: "qrc:/archive.svg"
                    width: 32
                    height: 32
                    fillMode: Image.PreserveAspectFit
                }
            }

            Rectangle {
                height: 50
                radius: 15
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignCenter

                color: "#211d44"

                Image {
                    anchors.centerIn: parent

                    source: "qrc:/upload.svg"
                    width: 32
                    height: 32
                    fillMode: Image.PreserveAspectFit
                }
            }
        }
    }

    Rectangle {
        id: pageRoot

        anchors.top:    parent.top
        anchors.bottom: parent.bottom
        anchors.left:   menuRoot.right
        anchors.right:  parent.right

        color: "#171430"

        property color boxColor: "#211d44"
        property var values: [
            ["Auftragsnummer", "55537"],
            ["Bearbeitet von", "12345"],
            ["Gestartet um", "10:30"],
            ["Beendet um", "-"],
            ["Gezählte Platten", "302"],
        ]

        RowLayout {
            id: info

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right

            anchors.topMargin: 20
            anchors.leftMargin: 12
            anchors.rightMargin: 20

            height: 80
            spacing: 12

            Repeater {
                model: pageRoot.values

                InfoBox {
                    required property var modelData
                    required property var index

                    backgroundColor: pageRoot.boxColor

                    descriptionText:  modelData[0]
                    descriptionColor: "#5c5890"

                    infoText:  modelData[1]
                    infoColor: "#dedcf8"

                    //width:  200
                    height: 80

                    Layout.fillWidth: true
                }
            }
        }

        Rectangle {
            id: plateOverView
            width: parent.width
            height: 150
            color: "blue"
            radius: 10
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: 20
            clip: true  // Hide overflow

            visible: false

            // Generate 100 random values in the range 9.0 to 11.6
            property var values: generateRandomValues(100, 9.0, 11.6)
            property real barWidth: 25

            function generateRandomValues(count, min, max) {
                var result = []
                for (let i = 0; i < count; i++) {
                    let value = Math.random() * (max - min) + min
                    result.push(value)
                }
                return result
            }

            // Flickable container for scrolling
            Flickable {
                id: flickable

                anchors.fill: parent

                clip: true

                contentWidth:  rowLayout.width  // Make the content width the same as the total bar width
                contentHeight: parent.height

                boundsMovement: Flickable.StopAtBounds
                // boundsBehavior: Flickable.DragOverBounds
                // opacity: Math.max(0.5, 1.0 - Math.abs(horizontalOvershoot) / width)

                flickableDirection: Flickable.HorizontalFlick

                MouseArea {
                    anchors.fill: parent

                    onWheel: (wheel) => {
                        var scaleFactor = wheel.angleDelta.y > 0 ? 1.2 : 0.8;

                        plateOverView.barWidth = Math.max(5, plateOverView.barWidth * scaleFactor);

                        // Recalculate contentWidth based on updated barWidth
                        plateOverView.width = plateOverView.values.length * plateOverView.barWidth + rowLayout.spacing * (plateOverView.values.length - 1)
                    }
                }

                RowLayout {
                    id: rowLayout
                    anchors.left: parent.left
                    anchors.bottom: parent.bottom

                    anchors.leftMargin:  5
                    anchors.rightMargin: 5

                    Repeater {
                        model: plateOverView.values

                        Rectangle {
                            id: bar
                            required property var modelData
                            required property var index
                            Layout.alignment: Qt.AlignBottom

                            width: plateOverView.barWidth
                            height: (modelData - 8.0) * 40
                            color: modelData >= 10.8 && modelData <= 11.2 ? "#a6da95" : "#ed8796"
                            topLeftRadius: 3
                            topRightRadius: 3
                        }
                    }
                }
            }
        }

        Rectangle {
            id: chart

            anchors.top: info.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom

            anchors.topMargin: 12
            anchors.bottomMargin: 20
            anchors.rightMargin: 20
            anchors.leftMargin: 12

            color: "#211d44"
            radius: 10

            MouseArea {
                anchors.fill: parent

                onWheel: (wheel) => {
                    var scaleFactor = wheel.angleDelta.y > 0 ? 1.2 : 0.8;
                    xAxis.zoom *= scaleFactor
                    yAxis.zoom *= scaleFactor
                }
            }

            GraphsView {
                id: graph

                anchors.fill: parent

                panStyle: GraphsView.PanStyle.Drag

                theme: GraphsTheme {
                    backgroundVisible: false
                    plotAreaBackgroundVisible: false

                    gridVisible: false

                    labelTextColor: "#dedcf8"

                    axisX.mainColor: "#dedcf8"
                    axisY.mainColor: "#dedcf8"
                }

                axisX: DateTimeAxis {
                    id: xAxis

                    labelFormat: "hh:mm:ss"

                    min: new Date(Date.now() - 0.1 * 60 * 60 * 1000)
                    max: new Date()

                    tickInterval: 12.0
                    subTickCount: 4.0

                    onZoomChanged: (x) => {
                        zoom = Math.max(Math.min(x, 60), 0.05)
                    }
                }

                axisY: ValueAxis {
                    id: yAxis

                    min: 0
                    max: 25.0

                    tickInterval: 5.0
                    subTickCount: 5.0
                    labelDecimals: 2

                    zoom: 1

                    function snapZoom(x) {
                        const exp = Math.floor(Math.log10(x))
                        const base = Math.pow(10, exp)
                        const norm = x / base

                        let snapped
                        if (norm < 1.5)
                            snapped = 1
                        else if (norm < 3.5)
                            snapped = 2
                        else
                            snapped = 5

                        return snapped * base
                    }

                    onZoomChanged: (x) => {
                        const clamped = Math.max(Math.min(x, 50), 0.05)
                        const snappedZoom = snapZoom(clamped)
                        tickInterval = 5 / snappedZoom
                        zoom = clamped
                    }
                }

                component Marker : Rectangle {
                    width: 16
                    height: 16
                    color: "#ffffff"
                    radius: width * 0.5
                    border.width: 4
                    border.color: "#000000"
                }

                LineSeries {
                    id: lineSeries1
                    width: 4
                    pointDelegate: Marker { }
                    XYPoint { x: 0; y: 0 }
                    XYPoint { x: 1; y: 2.1 }
                    XYPoint { x: 2; y: 3.3 }
                    XYPoint { x: 3; y: 2.1 }
                    XYPoint { x: 4; y: 4.9 }
                    XYPoint { x: 5; y: 3.0 }
                }

                LineSeries {
                    id: lineSeries2
                    width: 4
                    pointDelegate: Marker { }
                    XYPoint { x: 0; y: 5.0 }
                    XYPoint { x: 1; y: 3.3 }
                    XYPoint { x: 2; y: 7.1 }
                    XYPoint { x: 3; y: 7.5 }
                    XYPoint { x: 4; y: 6.1 }
                    XYPoint { x: 5; y: 3.2 }
                }
            }
        }
    }

    Rectangle {
        id:  exportBtn

        visible: false

        anchors.right: parent.right
        anchors.bottom: parent.bottom

        anchors.margins: 35

        width: 50
        height: 50

        color: "#5b39db"
        opacity: 0.25
    }
}
