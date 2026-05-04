cmake_minimum_required(VERSION 4.0)

project(MyQtApp VERSION 1.0)

find_package(Qt6 REQUIRED COMPONENTS Quick Qml Graphs)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_AUTORCC ON)

qt_add_executable(
    ${PROJECT_NAME}
    resources.qrc
    main.cpp
)

target_link_libraries(${PROJECT_NAME} 
    PRIVATE
     Qt6::Core 
     Qt6::Quick 
     Qt6::Qml 
     Qt6::Graphs
)