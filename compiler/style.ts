import { IStyle } from '../parser/interfaces/IStyle.ts'

export class CompileStyle {
    private output = "";
    private styles: IStyle[] = [];
    constructor (styleAST: IStyle[]){
        this.styles = styleAST;
    }

    compile() {
        
    }
    
}