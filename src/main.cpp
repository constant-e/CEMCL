#include "CEMCL.hpp"

#include <QApplication>

int main(int argc, char *argv[]) {
    QApplication a(argc, argv);
    a.setWindowIcon(QIcon(":/pic/icon.jpg"));
    CEMCL w;
    w.show();
    return a.exec();
}
