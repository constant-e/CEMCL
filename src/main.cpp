#include "CEMCL.hpp"

#include <QApplication>

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);
    CEMCL w;
    w.show();
    return a.exec();
}
